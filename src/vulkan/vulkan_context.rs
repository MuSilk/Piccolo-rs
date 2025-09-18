use std::{cell::RefCell, collections::{HashMap, HashSet}, ffi::CStr, os::raw::c_void, rc::Rc, time::Instant, u64, usize};

use anyhow::{anyhow, Result};
use log::*;
use nalgebra_glm::{Vec3};
use thiserror::Error;
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY}, prelude::v1_0::*, vk::{ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension}, window as vk_window, Version
};
use winit::{keyboard::KeyCode, window::Window};

use crate::{surface::{InstanceManager, Mesh, MeshManager, RenderInstance, TexturedMeshVertex}, utils::{Camera, CameraMovement}, vulkan::{create_image, create_image_view, pipeline::{PipelineManager, UniformBufferObject}, Destroy, Image, Pipeline, Texture, TextureManager, VulkanData}};

const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

const MAX_FRAMES_IN_FLIGHT: usize = 2;

const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

#[derive(Debug)]
pub struct VulkanContext {
    _entry: Entry,
    instance: Instance,
    data: VulkanData,
    device: Device,
    frame: usize,
    image_index: usize,
    pub resized: bool,

    last: Instant,

    camera: Camera,
    texture_manager: TextureManager,
    mesh_manager: MeshManager,
    pipeline_manager: PipelineManager,
    instance_manager: InstanceManager,
    key_pressed: HashSet<KeyCode>,
}

impl VulkanContext {

    pub fn create(window: &Window) -> Result<Self> {
        let (entry, instance, mut data, device) = create_environment(window)?;
        let mut pipeline_manager = PipelineManager::new();
        let pipeline = Pipeline::create_pipeline::<TexturedMeshVertex>(&device,&instance, &data)?;
        
        let mut texture_manager = HashMap::new();
        let texture = Texture::new(&instance, &device, &mut data, "resources/viking_room.png")?;
        
        pipeline.update_descriptor_sets(&device, texture.image_view, data.texture_sampler)?;

        let mut mesh_manager = HashMap::new();
        let mut model = Mesh::<TexturedMeshVertex>::eval_model("resources/viking_room.obj")?;
        model.build(&instance, &device, &data)?;

        let mut instance_manager = HashMap::new();

        let model = Rc::new(RefCell::new(model));

        let object = RenderInstance::new(&model);

        texture_manager.insert("viking_room", texture);
        pipeline_manager.insert("test_pipeline", pipeline);
        mesh_manager.insert("viking_room", model);
        instance_manager.insert("viking_room", object);

        let mut camera = Camera::new(&Vec3::new(2.0, 2.0, 2.0));
        camera.set_target(&Vec3::new(0.0, 0.0, 0.0));

        Ok(Self {
            last: Instant::now(),
            _entry: entry, instance, data, device, 
            frame: 0, image_index: 0, resized: false, 
            camera: camera,
            texture_manager, pipeline_manager, mesh_manager, instance_manager,
            key_pressed: HashSet::new(),
        })
    }

    pub fn render(&mut self,window: &Window) -> Result<()> {
        if let Ok(usize::MAX) = self.acquire_next_image(window) {
            return Ok(());
        }
        self.update(window)?;
        self.update_render_resources()?;

        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[self.image_index as usize]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        unsafe {
            self.device.reset_fences(&[self.data.in_flight_fences[self.frame]])?;
            self.device.queue_submit(
                self.data.graphics_queue, 
                &[submit_info],
                self.data.in_flight_fences[self.frame],
            )?;
        }

        self.present_frame(window, signal_semaphores)?;

        Ok(())
    }

    fn acquire_next_image(&mut self, window:&Window) -> Result<usize>{
        unsafe {
            self.device.wait_for_fences(
                &[self.data.in_flight_fences[self.frame]], 
                true, 
                u64::MAX
            )?;
        }

        let result = unsafe {
            self.device
            .acquire_next_image_khr(
                self.data.swapchain, 
                u64::MAX,
                self.data.image_available_semaphores[self.frame],
                vk::Fence::null()
            )
        }; 

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                self.recreate_swapchain(window)?;
                return Ok(usize::MAX);
            }
            Err(e) => return Err(anyhow!(e)),
        };

        if !self.data.images_in_flight[image_index].is_null() {
            unsafe {
                self.device.wait_for_fences(
                    &[self.data.images_in_flight[image_index]], 
                    true, 
                    u64::MAX
                )?;
            }
        }

        self.data.images_in_flight[image_index as usize] = self.data.in_flight_fences[self.frame];
        self.image_index = image_index;

        Ok(image_index)
    }

    fn present_frame(&mut self, window: &Window, signal_semaphores: &[vk::Semaphore]) -> Result<()>{
        let swapchains = &[self.data.swapchain];
        let image_indices = &[self.image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        unsafe {
            let result = self.device.queue_present_khr(self.data.present_queue, &present_info);
            let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR) || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
            if self.resized || changed {
                self.resized = false;
                self.recreate_swapchain(window)?;
            } else if let Err(e) = result {
                return Err(anyhow!(e));
            }
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }

    fn update_render_resources(&mut self) -> Result<()>{ 
        self.reset_command_pool()?;

        let instance = self.instance_manager.get("viking_room").ok_or(anyhow!("Instance not found"))?;
        let pipeline = self.pipeline_manager.get("test_pipeline").ok_or(anyhow!("Pipeline not found"))?;

        let model = instance.get_model_matrix();
        let view = self.camera.get_view_matrix();
        let proj = self.camera.get_projection_matrix(self.get_width(), self.get_height());
        let ubo = UniformBufferObject { model, view, proj };
        pipeline.set_uniform(&self.device, self.image_index, &ubo)?;

        let instance = self.instance_manager.get_mut("viking_room").ok_or(anyhow!("Instance not found"))?;

        let command_buffer = instance.update_command_buffer(&self.device, &self.data, pipeline, self.image_index)?;
        self.data.secondary_command_buffers = vec![command_buffer];

        self.update_command_buffer()?;
        Ok(())
    }

    fn reset_command_pool(&self) -> Result<()> {
        let command_pool = self.data.command_pools[self.image_index];
        unsafe{self.device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;}
        Ok(())
    }

    fn update_command_buffer(&self) -> Result<()> {
        let image_index = self.image_index as usize;
        let command_buffer = self.data.command_buffers[image_index];

        let inheritance = vk::CommandBufferInheritanceInfo::builder();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .inheritance_info(&inheritance);
        unsafe {
            self.device.begin_command_buffer(command_buffer, &info)?;
        }

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(self.data.swapchain_extent);

        let color_clear_value = vk::ClearValue{
            color: vk::ClearColorValue{float32 : [0.0, 0.0, 0.0, 1.0]}
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {depth:1.0, stencil: 0,}
        };

        let clear_values = &[color_clear_value, depth_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.data.render_pass)
            .framebuffer(self.data.framebuffers[image_index])
            .render_area(render_area)
            .clear_values(clear_values);
        unsafe {
            self.device.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS);
            self.device.cmd_execute_commands(command_buffer, &self.data.secondary_command_buffers);
            self.device.cmd_end_render_pass(command_buffer);
            self.device.end_command_buffer(command_buffer)?;
        }

        Ok(())
    }
        
    pub fn handle_key_press(&mut self, key: KeyCode){
        self.key_pressed.insert(key);
    }

    pub fn handle_key_release(&mut self, key: KeyCode){
        self.key_pressed.remove(&key);
    }

    pub fn handle_cursor_movement(&mut self, xoffset: f64, yoffset: f64){
        self.camera.process_mouse_movement(xoffset as f32, yoffset as f32, true);
    }
    pub fn update(&mut self, _window: &Window) -> Result<()> {

        let now = Instant::now();
        let delta_time = now.duration_since(self.last).as_secs_f32();
        self.last = now;

        let key_mapping = [
            (KeyCode::KeyW, CameraMovement::FORWARD),
            (KeyCode::KeyS, CameraMovement::BACKWARD),
            (KeyCode::KeyA, CameraMovement::LEFT),
            (KeyCode::KeyD, CameraMovement::RIGHT),
            (KeyCode::KeyQ, CameraMovement::DOWN),
            (KeyCode::KeyE, CameraMovement::UP),
        ];

        for (key, movement) in &key_mapping {
            if self.key_pressed.contains(key) {
                self.camera.process_keyboard(movement, delta_time);
            }
        }
        Ok(())
    }
    pub fn destroy(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.destroy_swapchain();
            self.pipeline_manager.destroy(&self.device);
            self.device.destroy_sampler(self.data.texture_sampler, None);
            self.texture_manager.destroy(&self.device);
            self.mesh_manager.destroy(&self.device);
            self.data.in_flight_fences.iter().for_each(|f| self.device.destroy_fence(*f, None));
            self.data.render_finished_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
            self.data.image_available_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
            self.device.destroy_command_pool(self.data.command_pool, None);
            self.data.command_pools.iter().for_each(|p| self.device.destroy_command_pool(*p, None));
            self.device.destroy_device(None);
            self.instance.destroy_surface_khr(self.data.surface, None);

            if VALIDATION_ENABLED {
                self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }

    fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        unsafe{self.device.device_wait_idle()?;}
        self.destroy_swapchain();
        create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
        create_swapchain_image_views(&self.device,&mut self.data)?;
        create_render_pass(&self.instance, &self.device, &mut self.data)?;
        create_color_objects(&self.instance, &self.device, &mut self.data)?;
        create_depth_objects(&self.instance, &self.device, &mut self.data)?;
        create_framebuffers(&self.device, &mut self.data)?;
        self.data
            .images_in_flight
            .resize(self.data.swapchain_images.len(), vk::Fence::null());
        
        self.pipeline_manager.values_mut().for_each(|pipeline| {
            let _ = pipeline.recreate::<TexturedMeshVertex>(&self.device, &self.instance,&self.data);
        });
        create_command_buffers(&self.device, &mut self.data)?;
        Ok(())
    }

    fn destroy_swapchain(&self){
        unsafe {
            self.data.color_image.destroy(&self.device);
            self.device.free_memory(self.data.color_image_memory, None);
            self.device.destroy_image_view(self.data.depth_image_view, None);
            self.device.free_memory(self.data.depth_image_memory, None);
            self.device.destroy_image(self.data.depth_image, None);
            self.data.framebuffers.iter().for_each(|f| self.device.destroy_framebuffer(*f, None));
            self.device.destroy_render_pass(self.data.render_pass, None);
            self.data.swapchain_image_views.iter().for_each(|v| self.device.destroy_image_view(*v, None));
            self.device.destroy_swapchain_khr(self.data.swapchain, None);
        }
    }

    fn get_width(&self) -> u32{
        self.data.swapchain_extent.width
    }

    fn get_height(&self) -> u32{
        self.data.swapchain_extent.height
    }

}

fn create_environment(window: &Window) -> Result<(Entry,Instance,VulkanData,Device)> {
    let mut data = VulkanData::default();
    let entry =  unsafe {
        let loader = LibloadingLoader::new(LIBRARY)?;
        Entry::new(loader).map_err(|e| anyhow!("{}", e))?
    };
    let instance = create_instance(&window, &entry, &mut data)?;
    data.surface = unsafe {
        vk_window::create_surface(&instance, &window, &window)?
    };
    pick_physical_device(&instance, &mut data)?;
    let device = create_logical_device(&entry, &instance, &mut data)?;
    create_swapchain(&window, &instance, &device, &mut data)?;
    create_swapchain_image_views(&device, &mut data)?;
    create_render_pass(&instance, &device, &mut data)?;
    create_color_objects(&instance, &device, &mut data)?;
    create_depth_objects(&instance, &device, &mut data)?;
    create_framebuffers(&device, &mut data)?;
    create_command_pools(&instance, &device, &mut data)?;
    create_command_buffers(&device, &mut data)?;
    create_texture_sampler(&device, &mut data)?;
    create_sync_objects(&device, &mut data)?;
    Ok((entry, instance, data, device))
}

fn create_instance(window: &Window, entry: &Entry,data: &mut VulkanData) -> Result<Instance> {
    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Vulkan Tutorial\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"No Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    let available_layers = unsafe {
        entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>()
    }; 

    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    }else{
        Vec::new()
    };

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    if VALIDATION_ENABLED {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    let flags = if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION{
        info!("Enabling extensions for macOS portability.");
        extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
        extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        vk::InstanceCreateFlags::empty()
    };

    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .flags(flags);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION)
        .user_callback(Some(debug_callback));

    if VALIDATION_ENABLED {
        info = info.push_next(&mut debug_info);
    }

    let instance = unsafe {
        entry.create_instance(&info, None)?
    };

    if VALIDATION_ENABLED {
        data.messenger = unsafe {
            instance.create_debug_utils_messenger_ext(&debug_info, None)?
        };
    }

    Ok(instance)
}

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,   
) -> vk::Bool32 {
    let data = unsafe {*data};
    let message = unsafe { CStr::from_ptr(data.message)}.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    }else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    }else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    }else{
        trace!("({:?}) {}", type_, message);
    }

    vk::FALSE
}

#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub struct SuitabilityError(pub &'static str);
fn pick_physical_device(instance: &Instance, data: &mut VulkanData) -> Result<()> {
    unsafe {
        for physical_device in instance.enumerate_physical_devices()?{
            let properties = instance.get_physical_device_properties(physical_device);

            if let Err(error) = check_physical_device(instance, data, physical_device) {
                warn!("Skipping physical device (`{}`): {}", properties.device_name, error);
            } else {
                info!("Selected physical device (`{}`).", properties.device_name);
                data.physical_device = physical_device;
                data.msaa_samples = get_max_msaa_samples(instance, data.physical_device);
                return Ok(());
            }
        }
    }

    Err(anyhow!("Failed to find suitable physical device."))
}
fn check_physical_device(
    instance: &Instance,
    data: &VulkanData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    QueueFamilyIndices::get(instance, data, physical_device)?;
    check_physical_device_extensions(instance, physical_device)?;

    let support = SwapchainSupport::get(instance, data, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!(SuitabilityError("Insufficient swapchain support.")));
    }

    let features = unsafe{ instance.get_physical_device_features(physical_device) };
    if features.sampler_anisotropy != vk::TRUE {
        return Err(anyhow!(SuitabilityError("Anisotropic filtering is not supported.")));
    }

    Ok(())
}

fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {   
    let extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device, None)?
            .iter()
            .map(|e| e.extension_name)
            .collect::<HashSet<_>>()
    };

    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)){
        Ok(())
    }else{
        Err(anyhow!(SuitabilityError("Missing required device extensions.")))
    }
}

#[derive(Copy, Clone, Debug)]
struct QueueFamilyIndices {
    graphics: u32,
    present: u32,
}

impl QueueFamilyIndices {
    fn get(
        instance: &Instance,
        data: &VulkanData,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> { 
        let properties = unsafe {
            instance.get_physical_device_queue_family_properties(physical_device)
        };
        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        let mut present = None;
        for (index,_properties) in properties.iter().enumerate() {
            if unsafe {
                instance.get_physical_device_surface_support_khr(physical_device, index as u32, data.surface)?
            } {
                present = Some(index as u32);
                break;
            }
        }

        if let (Some(graphics), Some(present)) = (graphics, present) {
            Ok(Self { graphics, present })
        } else {
            Err(anyhow!(SuitabilityError("graphics queue family")))
        }
    }
}

fn create_logical_device(
    entry: &Entry,
    instance: &Instance,
    data: &mut VulkanData,
) -> Result<Device> { 
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices.
        iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        Vec::new()
    };

    let mut extensions = DEVICE_EXTENSIONS.
        iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();
    if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
    }

    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .sample_rate_shading(true);

    let device_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = unsafe {
        instance.create_device(data.physical_device, &device_info, None)?
    };

    unsafe {
        data.graphics_queue = device.get_device_queue(indices.graphics, 0);
        data.present_queue = device.get_device_queue(indices.present, 0);
    }

    Ok(device)
}

#[derive(Clone, Debug)]
struct SwapchainSupport {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    fn get(
        instance: &Instance,
        data: &VulkanData,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> {
        unsafe {
            Ok(Self { 
                capabilities: instance.
                    get_physical_device_surface_capabilities_khr(physical_device, data.surface)?,
                formats: instance.
                    get_physical_device_surface_formats_khr(physical_device, data.surface)?,
                present_modes: instance.
                    get_physical_device_surface_present_modes_khr(physical_device, data.surface)?,
            })
        }
    }
}

fn get_swapchain_surface_format(
    formats: &[vk::SurfaceFormatKHR],
) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .cloned()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
            && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| formats[0])
}

fn get_swapchain_present_mode(
    present_modes: &[vk::PresentModeKHR],
) -> vk::PresentModeKHR {
    present_modes
        .iter()
        .cloned()
        .find(|&m| m == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

fn get_swapchain_extent(
    window: &Window,
    capabilities: vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        vk::Extent2D::builder()
            .width(window.inner_size().width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ))
            .height(window.inner_size().height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ))
            .build()
    }
}

fn create_swapchain(
    window: &Window,
    instance: &Instance,
    device: &Device,
    data: &mut VulkanData,
) -> Result<()> {
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;
    let support = SwapchainSupport::get(instance, data, data.physical_device)?;

    let surface_format = get_swapchain_surface_format(&support.formats);
    let present_mode = get_swapchain_present_mode(&support.present_modes);
    let extent = get_swapchain_extent(window, support.capabilities);

    let mut image_count = support.capabilities.min_image_count + 1;
    if support.capabilities.max_image_count != 0 
        && image_count > support.capabilities.max_image_count 
    {
        image_count = support.capabilities.max_image_count;
    }

    let mut queue_family_indices = vec![];
    let image_sharing_mode = if indices.graphics != indices.present {
        queue_family_indices.push(indices.graphics);
        queue_family_indices.push(indices.present);
        vk::SharingMode::CONCURRENT
    } else {
        vk::SharingMode::EXCLUSIVE
    };
    
    let info = vk::SwapchainCreateInfoKHR::builder()
        .surface(data.surface)
        
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());

    data.swapchain_format = surface_format.format;
    data.swapchain_extent = extent; 
    unsafe {
        data.swapchain = device.create_swapchain_khr(&info, None)?;
        data.swapchain_images = device.get_swapchain_images_khr(data.swapchain)?;
        info!("swapchain images: {}",data.swapchain_images.len());
    }

    Ok(())
}

fn create_swapchain_image_views(device: &Device, data: &mut VulkanData) -> Result<()> {

    data.swapchain_image_views = data
        .swapchain_images
        .iter()
        .map(|i| create_image_view(device, *i, data.swapchain_format,vk::ImageAspectFlags::COLOR, 1))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(())
}

fn create_render_pass(instance: &Instance, device: &Device, data: &mut VulkanData) -> Result<()> {

    let color_attachment =vk::AttachmentDescription::builder()
        .format(data.swapchain_format)
        .samples(data.msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];

    let depth_stencil_attachment = vk::AttachmentDescription::builder()
        .format(get_depth_format(instance, data)?)
        .samples(data.msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let color_resolve_attachment = vk::AttachmentDescription::builder()
        .format(data.swapchain_format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_resolve_attachment_ref = vk::AttachmentReference::builder()
        .attachment(2)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let resolve_attachments = &[color_resolve_attachment_ref];

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments)
        .depth_stencil_attachment(&depth_stencil_attachment_ref)
        .resolve_attachments(resolve_attachments);

    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE
            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

    let attachments = &[color_attachment, depth_stencil_attachment, color_resolve_attachment];
    let subpasses = &[subpass];
    let dependencies = &[dependency];
    let info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);

    data.render_pass = unsafe { device.create_render_pass(&info, None)? };

    Ok(())
}

fn create_framebuffers(device: &Device, data: &mut VulkanData) -> Result<()> {
    data.framebuffers = data
        .swapchain_image_views
        .iter()
        .map(|i| {
            let attachments = &[data.color_image.image_view,data.depth_image_view,*i];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(data.render_pass)
                .attachments(attachments)
                .width(data.swapchain_extent.width)
                .height(data.swapchain_extent.height)
                .layers(1);

            unsafe { device.create_framebuffer(&create_info, None) }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

fn create_command_pools(instance: &Instance, device: &Device, data: &mut VulkanData) -> Result<()> {
    data.command_pool = create_command_pool(instance, device, data)?;
    for _ in 0..data.framebuffers.len() {
        let command_pool = create_command_pool(instance, device, data)?;
        data.command_pools.push(command_pool);
    }
    Ok(())
}

fn create_command_pool(instance: &Instance, device: &Device, data: &mut VulkanData) -> Result<vk::CommandPool> {
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(indices.graphics);
    unsafe {
        Ok(device.create_command_pool(&info, None)?) 
    }
}

fn create_command_buffers(device: &Device, data: &mut VulkanData) -> Result<()> {
    let num_images = data.swapchain_images.len();
    for image_index in 0..num_images {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(data.command_pools[image_index])
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe { device.allocate_command_buffers(&allocate_info)?[0] };
        data.command_buffers.push(command_buffer);
    }
    Ok(())
}

fn create_sync_objects(device: &Device, data: &mut VulkanData) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED);

    unsafe {
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            data.image_available_semaphores
                .push(device.create_semaphore(&semaphore_info, None)?);
            data.render_finished_semaphores
                .push(device.create_semaphore(&semaphore_info, None)?);

            data.in_flight_fences
                .push(device.create_fence(&fence_info, None)?);
        }
    }

    data.images_in_flight = data.swapchain_images
        .iter()
        .map(|_| vk::Fence::null())
        .collect();

    Ok(())
}

fn create_texture_sampler(device: &Device, data: &mut VulkanData) -> Result<()> {
    let sampler_info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(true)
        .max_anisotropy(16.0)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .min_lod(0.0)
        .max_lod(vk::LOD_CLAMP_NONE)
        .mip_lod_bias(0.0);

    data.texture_sampler = unsafe {
        device.create_sampler(&sampler_info, None)?
    };

    Ok(())
}

fn create_depth_objects(instance: &Instance, device: &Device, data: &mut VulkanData) -> Result<()> {

    let format = get_depth_format(instance, data)?;

    let (depth_image, depth_image_memory) = create_image(
        instance, device, data, 
        data.swapchain_extent.width,
        data.swapchain_extent.height,
        1,
        data.msaa_samples,
        format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.depth_image = depth_image;
    data.depth_image_memory = depth_image_memory;

    data.depth_image_view = create_image_view(device, data.depth_image, format,vk::ImageAspectFlags::DEPTH,1)?;

    Ok(())
}

fn get_supported_format(
    instance: &Instance,
    data: &VulkanData,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> Result<vk::Format> {
    candidates
        .iter()
        .cloned()
        .find(|f|{
            let properties = unsafe {
                instance.get_physical_device_format_properties(data.physical_device, *f)
            };
            match tiling {
                vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                _ => false,
            }
        })
        .ok_or_else(|| anyhow!("Failed to find supported format!"))
}

fn get_depth_format(instance: &Instance, data: &VulkanData) -> Result<vk::Format>{
    let candidates = &[
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    get_supported_format(
        instance, data, candidates, 
        vk::ImageTiling::OPTIMAL, 
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
    )
}

fn get_max_msaa_samples(instance: &Instance, physical_device: vk::PhysicalDevice) -> vk::SampleCountFlags {
    let properties = unsafe {instance.get_physical_device_properties(physical_device)};
    let counts = properties.limits.framebuffer_color_sample_counts & 
        properties.limits.framebuffer_depth_sample_counts;

    [
        vk::SampleCountFlags::_64,
        vk::SampleCountFlags::_32,
        vk::SampleCountFlags::_16,
        vk::SampleCountFlags::_8,
        vk::SampleCountFlags::_4,
        vk::SampleCountFlags::_2,
    ].iter()
    .cloned()
    .find(|c| counts.contains(*c))
    .unwrap_or(vk::SampleCountFlags::_1)
}

fn create_color_objects(instance: &Instance, device: &Device, data: &mut VulkanData)->Result<()> {
    let (color_image, color_image_memory) = create_image(
        instance,
        device,
        data,
        data.swapchain_extent.width,
        data.swapchain_extent.height,
        1,
        data.msaa_samples,
        data.swapchain_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT
            | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.color_image_memory = color_image_memory;

    let image_view = create_image_view(
        device,
        color_image,
        data.swapchain_format,
        vk::ImageAspectFlags::COLOR,
        1,
    )?;

    data.color_image = Image { image:color_image, image_view };

    Ok(())
}