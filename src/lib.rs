#![feature(unsafe_destructor)]
#![feature(globs)]
#![feature(phase)]
#![unstable]

//! The purpose of this library is to provide an OpenGL context on as many
//!  platforms as possible.
//!
//! # Building a window
//!
//! There are two ways to create a window:
//!
//!  - Calling `Window::new()`.
//!  - Calling `let builder = WindowBuilder::new()` then `builder.build()`.
//!
//! The first way is the simpliest way and will give you default values.
//!
//! The second way allows you to customize the way your window and GL context
//!  will look and behave.
//!
//! # Features
//!
//! This crate has two Cargo features: `window` and `headless`.
//!
//!  - `window` allows you to create regular windows and enables the `WindowBuilder` object.
//!  - `headless` allows you to do headless rendering, and enables
//!     the `HeadlessRendererBuilder` object.
//!
//! By default only `window` is enabled.

extern crate gl_common;
extern crate libc;

#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "macos")]
extern crate cocoa;
#[cfg(target_os = "macos")]
extern crate core_foundation;
#[cfg(target_os = "macos")]
extern crate core_graphics;

pub use events::*;

use std::default::Default;

#[cfg(all(not(target_os = "windows"), not(target_os = "linux"), not(target_os = "macos"), not(target_os = "android")))]
use this_platform_is_not_supported;

#[cfg(target_os = "windows")]
#[path="win32/mod.rs"]
mod winimpl;
#[cfg(target_os = "linux")]
#[path="x11/mod.rs"]
mod winimpl;
#[cfg(target_os = "macos")]
#[path="osx/mod.rs"]
mod winimpl;
#[cfg(target_os = "android")]
#[path="android/mod.rs"]
mod winimpl;

mod events;

/// Identifier for a monitor.
#[cfg(feature = "window")]
pub struct MonitorID(winimpl::MonitorID);

/// Error that can happen while creating a window or a headless renderer.
#[deriving(Clone, Show, PartialEq, Eq)]
pub enum CreationError {
    OsError(String),
}

impl std::error::Error for CreationError {
    fn description(&self) -> &str {
        match self {
            &CreationError::OsError(ref text) => text.as_slice(),
        }
    }
}

/// All APIs related to OpenGL that you can possibly get while using glutin.
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
pub enum Api {
    /// The classical OpenGL. Available on Windows, Linux, OS/X.
    OpenGl,
    /// OpenGL embedded system. Available on Linux, Android.
    OpenGlEs,
}

/// Object that allows you to build windows.
#[cfg(feature = "window")]
pub struct WindowBuilder<'a> {
    sharing: Option<&'a Window>,
    dimensions: Option<(uint, uint)>,
    title: String,
    monitor: Option<winimpl::MonitorID>,
    gl_version: Option<(uint, uint)>,
    gl_debug: bool,
    vsync: bool,
    visible: bool,
    multisampling: Option<u16>,
}

#[cfg(feature = "window")]
impl<'a> WindowBuilder<'a> {
    /// Initializes a new `WindowBuilder` with default values.
    pub fn new() -> WindowBuilder<'a> {
        WindowBuilder {
            sharing: None,
            dimensions: None,
            title: "glutin window".to_string(),
            monitor: None,
            gl_version: None,
            gl_debug: cfg!(ndebug),
            vsync: false,
            visible: true,
            multisampling: None,
        }
    }

    /// Requests the window to be of specific dimensions.
    ///
    /// Width and height are in pixels.
    pub fn with_dimensions(mut self, width: uint, height: uint) -> WindowBuilder<'a> {
        self.dimensions = Some((width, height));
        self
    }

    /// Requests a specific title for the window.
    pub fn with_title(mut self, title: String) -> WindowBuilder<'a> {
        self.title = title;
        self
    }

    /// Requests fullscreen mode.
    ///
    /// If you don't specify dimensions for the window, it will match the monitor's.
    pub fn with_fullscreen(mut self, monitor: MonitorID) -> WindowBuilder<'a> {
        let MonitorID(monitor) = monitor;
        self.monitor = Some(monitor);
        self
    }

    /// The created window will share all its OpenGL objects with the window in the parameter.
    ///
    /// There are some exceptions, like FBOs or VAOs. See the OpenGL documentation.
    pub fn with_shared_lists(mut self, other: &'a Window) -> WindowBuilder<'a> {
        self.sharing = Some(other);
        self
    }

    /// Requests to use a specific OpenGL version.
    ///
    /// Version is a (major, minor) pair. For example to request OpenGL 3.3
    ///  you would pass `(3, 3)`.
    pub fn with_gl_version(mut self, version: (uint, uint)) -> WindowBuilder<'a> {
        self.gl_version = Some(version);
        self
    }

    /// Sets the *debug* flag for the OpenGL context.
    ///
    /// The default value for this flag is `cfg!(ndebug)`, which means that it's enabled
    /// when you run `cargo build` and disabled when you run `cargo build --release`.
    pub fn with_gl_debug_flag(mut self, flag: bool) -> WindowBuilder<'a> {
        self.gl_debug = flag;
        self
    }

    /// Requests that the window has vsync enabled.
    pub fn with_vsync(mut self) -> WindowBuilder<'a> {
        self.vsync = true;
        self
    }

    /// Sets whether the window will be initially hidden or visible.
    pub fn with_visibility(mut self, visible: bool) -> WindowBuilder<'a> {
        self.visible = visible;
        self
    }

    /// Sets the multisampling level to request.
    ///
    /// # Panic
    ///
    /// Will panic if `samples` is not a power of two.
    pub fn with_multisampling(mut self, samples: u16) -> WindowBuilder<'a> {
        use std::num::UnsignedInt;
        assert!(samples.is_power_of_two());
        self.multisampling = Some(samples);
        self
    }

    /// Builds the window.
    ///
    /// Error should be very rare and only occur in case of permission denied, incompatible system,
    ///  out of memory, etc.
    pub fn build(mut self) -> Result<Window, CreationError> {
        // resizing the window to the dimensions of the monitor when fullscreen
        if self.dimensions.is_none() && self.monitor.is_some() {
            self.dimensions = Some(self.monitor.as_ref().unwrap().get_dimensions())
        }

        // default dimensions
        if self.dimensions.is_none() {
            self.dimensions = Some((1024, 768));
        }

        // building
        winimpl::Window::new(self).map(|w| Window { window: w })
    }
}

/// Object that allows you to build headless contexts.
#[cfg(feature = "headless")]
pub struct HeadlessRendererBuilder {
    dimensions: (uint, uint),
    gl_version: Option<(uint, uint)>,
    gl_debug: bool,
}

#[cfg(feature = "headless")]
impl HeadlessRendererBuilder {
    /// Initializes a new `HeadlessRendererBuilder` with default values.
    pub fn new(width: uint, height: uint) -> HeadlessRendererBuilder {
        HeadlessRendererBuilder {
            dimensions: (width, height),
            gl_version: None,
            gl_debug: cfg!(ndebug),
        }
    }

    /// Requests to use a specific OpenGL version.
    ///
    /// Version is a (major, minor) pair. For example to request OpenGL 3.3
    ///  you would pass `(3, 3)`.
    pub fn with_gl_version(mut self, version: (uint, uint)) -> HeadlessRendererBuilder {
        self.gl_version = Some(version);
        self
    }

    /// Sets the *debug* flag for the OpenGL context.
    ///
    /// The default value for this flag is `cfg!(ndebug)`, which means that it's enabled
    /// when you run `cargo build` and disabled when you run `cargo build --release`.
    pub fn with_gl_debug_flag(mut self, flag: bool) -> HeadlessRendererBuilder {
        self.gl_debug = flag;
        self
    }

    /// Builds the headless context.
    ///
    /// Error should be very rare and only occur in case of permission denied, incompatible system,
    ///  out of memory, etc.
    pub fn build(self) -> Result<HeadlessContext, CreationError> {
        winimpl::HeadlessContext::new(self).map(|w| HeadlessContext { context: w })
    }
}

/// Represents an OpenGL context and the Window or environment around it.
///
/// # Example
///
/// ```ignore
/// let window = Window::new().unwrap();
///
/// unsafe { window.make_current() };
///
/// loop {
///     for event in window.poll_events() {
///             // process events here
///             _ => ()
///         }
///     }
///
///     // draw everything here
///
///     window.swap_buffers();
///     std::io::timer::sleep(17);
/// }
/// ```
#[cfg(feature = "window")]
pub struct Window {
    window: winimpl::Window,
}

#[cfg(feature = "window")]
impl Default for Window {
    fn default() -> Window {
        Window::new().unwrap()
    }
}

#[cfg(feature = "window")]
impl Window {
    /// Creates a new OpenGL context, and a Window for platforms where this is appropriate.
    ///
    /// This function is equivalent to `WindowBuilder::new().build()`.
    ///
    /// Error should be very rare and only occur in case of permission denied, incompatible system,
    ///  out of memory, etc.
    #[inline]
    #[cfg(feature = "window")]
    pub fn new() -> Result<Window, CreationError> {
        let builder = WindowBuilder::new();
        builder.build()
    }

    /// Returns true if the window has previously been closed by the user.
    #[inline]
    pub fn is_closed(&self) -> bool {
        self.window.is_closed()
    }

    /// Returns true if the window has previously been closed by the user.
    #[inline]
    #[deprecated = "Use is_closed instead"]
    pub fn should_close(&self) -> bool {
        self.is_closed()
    }

    /// Modifies the title of the window.
    ///
    /// This is a no-op if the window has already been closed.
    #[inline]
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title)
    }

    /// Shows the window if it was hidden.
    ///
    /// ## Platform-specific
    ///
    /// - Has no effect on Android
    ///
    #[inline]
    pub fn show(&self) {
        self.window.show()
    }

    /// Hides the window if it was visible.
    ///
    /// ## Platform-specific
    ///
    /// - Has no effect on Android
    ///
    #[inline]
    pub fn hide(&self) {
        self.window.hide()
    }

    /// Returns the position of the top-left hand corner of the window relative to the
    ///  top-left hand corner of the desktop.
    ///
    /// Note that the top-left hand corner of the desktop is not necessarly the same as
    ///  the screen. If the user uses a desktop with multiple monitors, the top-left hand corner
    ///  of the desktop is the top-left hand corner of the monitor at the top-left of the desktop.
    ///
    /// The coordinates can be negative if the top-left hand corner of the window is outside
    ///  of the visible screen region.
    ///
    /// Returns `None` if the window no longer exists.
    #[inline]
    pub fn get_position(&self) -> Option<(int, int)> {
        self.window.get_position()
    }

    /// Modifies the position of the window.
    ///
    /// See `get_position` for more informations about the coordinates.
    ///
    /// This is a no-op if the window has already been closed.
    #[inline]
    pub fn set_position(&self, x: int, y: int) {
        self.window.set_position(x, y)
    }

    /// Returns the size in pixels of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    /// These are the dimensions of the frame buffer, and the dimensions that you should use
    ///  when you call `glViewport`.
    ///
    /// Returns `None` if the window no longer exists.
    #[inline]
    pub fn get_inner_size(&self) -> Option<(uint, uint)> {
        self.window.get_inner_size()
    }

    /// Returns the size in pixels of the window.
    ///
    /// These dimensions include title bar and borders. If you don't want these, you should use
    ///  use `get_inner_size` instead.
    ///
    /// Returns `None` if the window no longer exists.
    #[inline]
    pub fn get_outer_size(&self) -> Option<(uint, uint)> {
        self.window.get_outer_size()
    }

    /// Modifies the inner size of the window.
    ///
    /// See `get_inner_size` for more informations about the values.
    ///
    /// This is a no-op if the window has already been closed.
    #[inline]
    pub fn set_inner_size(&self, x: uint, y: uint) {
        self.window.set_inner_size(x, y)
    }

    /// Returns an iterator to all the events that are currently in the window's events queue.
    ///
    /// Contrary to `wait_events`, this function never blocks.
    #[inline]
    pub fn poll_events(&self) -> PollEventsIterator {
        PollEventsIterator { data: self.window.poll_events() }
    }

    /// Waits for an event, then returns an iterator to all the events that are currently
    ///  in the window's events queue.
    ///
    /// If there are no events in queue when you call the function,
    ///  this function will block until there is one.
    #[inline]
    pub fn wait_events(&self) -> WaitEventsIterator {
        WaitEventsIterator { data: self.window.wait_events() }
    }

    /// Sets the context as the current context.
    #[inline]
    pub unsafe fn make_current(&self) {
        self.window.make_current()
    }

    /// Returns the address of an OpenGL function.
    ///
    /// Contrary to `wglGetProcAddress`, all available OpenGL functions return an address.
    #[inline]
    pub fn get_proc_address(&self, addr: &str) -> *const libc::c_void {
        self.window.get_proc_address(addr) as *const libc::c_void
    }

    /// Swaps the buffers in case of double or triple buffering.
    ///
    /// You should call this function every time you have finished rendering, or the image
    ///  may not be displayed on the screen.
    ///
    /// **Warning**: if you enabled vsync, this function will block until the next time the screen
    /// is refreshed. However drivers can choose to override your vsync settings, which means that
    /// you can't know in advance whether `swap_buffers` will block or not.
    #[inline]
    pub fn swap_buffers(&self) {
        self.window.swap_buffers()
    }

    /// Gets the native platform specific display for this window.
    /// This is typically only required when integrating with
    /// other libraries that need this information.
    #[inline]
    pub unsafe fn platform_display(&self) -> *mut libc::c_void {
        self.window.platform_display()
    }

    /// Returns the API that is currently provided by this window.
    ///
    /// - On Windows and OS/X, this always returns `OpenGl`.
    /// - On Android, this always returns `OpenGlEs`.
    /// - On Linux, it must be checked at runtime.
    pub fn get_api(&self) -> Api {
        self.window.get_api()
    }

    /// Create a window proxy for this window, that can be freely
    /// passed to different threads.
    #[inline]
    pub fn create_window_proxy(&self) -> WindowProxy {
        WindowProxy {
            proxy: self.window.create_window_proxy()
        }
    }

    /// Sets a resize callback that is called by Mac (and potentially other
    /// operating systems) during resize operations. This can be used to repaint
    /// during window resizing.
    #[experimental]
    pub fn set_window_resize_callback(&mut self, callback: Option<fn(uint, uint)>) {
        self.window.set_window_resize_callback(callback);
    }
}

#[cfg(feature = "window")]
impl gl_common::GlFunctionsSource for Window {
    fn get_proc_addr(&self, addr: &str) -> *const libc::c_void {
        self.get_proc_address(addr)
    }
}

/// Represents a thread safe subset of operations that can be called
/// on a window. This structure can be safely cloned and sent between
/// threads.
///
#[cfg(feature = "window")]
#[deriving(Clone)]
pub struct WindowProxy {
    proxy: winimpl::WindowProxy,
}

#[cfg(feature = "window")]
impl WindowProxy {

    /// Triggers a blocked event loop to wake up. This is
    /// typically called when another thread wants to wake
    /// up the blocked rendering thread to cause a refresh.
    #[inline]
    pub fn wakeup_event_loop(&self) {
        self.proxy.wakeup_event_loop();
    }
}

/// Represents a headless OpenGL context.
#[cfg(feature = "headless")]
pub struct HeadlessContext {
    context: winimpl::HeadlessContext,
}

#[cfg(feature = "headless")]
impl HeadlessContext {
    /// Creates a new OpenGL context
    /// Sets the context as the current context.
    #[inline]
    pub unsafe fn make_current(&self) {
        self.context.make_current()
    }

    /// Returns the address of an OpenGL function.
    ///
    /// Contrary to `wglGetProcAddress`, all available OpenGL functions return an address.
    #[inline]
    pub fn get_proc_address(&self, addr: &str) -> *const libc::c_void {
        self.context.get_proc_address(addr) as *const libc::c_void
    }

    /// Returns the API that is currently provided by this window.
    ///
    /// See `Window::get_api` for more infos.
    pub fn get_api(&self) -> Api {
        self.context.get_api()
    }

    #[experimental]
    pub fn set_window_resize_callback(&mut self, _: Option<fn(uint, uint)>) {
    }
}

#[cfg(feature = "headless")]
impl gl_common::GlFunctionsSource for HeadlessContext {
    fn get_proc_addr(&self, addr: &str) -> *const libc::c_void {
        self.get_proc_address(addr)
    }
}

/// An iterator for the `poll_events` function.
// Implementation note: we retreive the list once, then serve each element by one by one.
// This may change in the future.
pub struct PollEventsIterator<'a> {
    data: Vec<Event>,
}

impl<'a> Iterator<Event> for PollEventsIterator<'a> {
    fn next(&mut self) -> Option<Event> {
        self.data.remove(0)
    }
}

/// An iterator for the `wait_events` function.
// Implementation note: we retreive the list once, then serve each element by one by one.
// This may change in the future.
pub struct WaitEventsIterator<'a> {
    data: Vec<Event>,
}

impl<'a> Iterator<Event> for WaitEventsIterator<'a> {
    fn next(&mut self) -> Option<Event> {
        self.data.remove(0)
    }
}

/// An iterator for the list of available monitors.
// Implementation note: we retreive the list once, then serve each element by one by one.
// This may change in the future.
#[cfg(feature = "window")]
pub struct AvailableMonitorsIter {
    data: Vec<winimpl::MonitorID>,
}

#[cfg(feature = "window")]
impl Iterator<MonitorID> for AvailableMonitorsIter {
    fn next(&mut self) -> Option<MonitorID> {
        self.data.remove(0).map(|id| MonitorID(id))
    }
}

/// Returns the list of all available monitors.
#[cfg(feature = "window")]
pub fn get_available_monitors() -> AvailableMonitorsIter {
    let data = winimpl::get_available_monitors();
    AvailableMonitorsIter{ data: data }
}

/// Returns the primary monitor of the system.
#[cfg(feature = "window")]
pub fn get_primary_monitor() -> MonitorID {
    MonitorID(winimpl::get_primary_monitor())
}

#[cfg(feature = "window")]
impl MonitorID {
    /// Returns a human-readable name of the monitor.
    pub fn get_name(&self) -> Option<String> {
        let &MonitorID(ref id) = self;
        id.get_name()
    }

    /// Returns the number of pixels currently displayed on the monitor.
    pub fn get_dimensions(&self) -> (uint, uint) {
        let &MonitorID(ref id) = self;
        id.get_dimensions()
    }
}
