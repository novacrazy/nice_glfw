// Copyright 2014 The GLFW-RS Developers. For a full listing of the authors,
// refer to the AUTHORS file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Builder for a GLFW window with robust OpenGL context selection. See the `WindowBuilder` type.

extern crate glfw;
#[macro_use] extern crate log;

use std::sync::mpsc::Receiver;

use glfw::{Glfw, WindowMode, WindowHint, Window, WindowEvent, OpenGlProfileHint};

/// Builder for a GLFW window with robust OpenGL context selection.
///
/// Its lifetime paramters correspond to different attributes:
///
/// - `'glfw`: The `&'glfw Glfw` value the `WindowBuilder` got constructed with.
/// - `'title`: The `&'title str` choosen as an title, if any.
/// - `'monitor`: The `WindowMode<'monitor>` choosen for the window, if any.
///
/// # Example
///
/// ```rust,no_run
/// extern crate glfw;
/// extern crate nice_glfw;
///
/// fn main() {
///     let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
///     let window = nice_glfw::WindowBuilder::new(&mut glfw)
///         .try_modern_context_hints()
///         .size(800, 600)
///         .create();
///
///     // ... rest of program
/// }
/// ```
pub struct WindowBuilder<'glfw, 'title, 'monitor> {
    glfw: &'glfw mut Glfw,
    size: Option<(u32, u32)>,
    aspect_ratio: Option<(u32, u32)>,
    title: Option<&'title str>,
    mode: Option<WindowMode<'monitor>>,
    common_hints: Vec<WindowHint>,
    try_hints: Vec<Vec<WindowHint>>,
}

impl<'glfw, 'title, 'monitor> WindowBuilder<'glfw, 'title, 'monitor> {
    /// Creates a new `WindowBuilder` for a existing `Glfw` value
    pub fn new(glfw: &'glfw mut Glfw) -> WindowBuilder<'glfw, 'title, 'monitor> {
        WindowBuilder {
            glfw: glfw,
            size: None,
            aspect_ratio: None,
            title: None,
            mode: None,
            try_hints: vec![],
            common_hints: vec![],
        }
    }
}

impl<'glfw, 'title, 'monitor, 'hints> WindowBuilder<'glfw, 'title, 'monitor> {
    /// Sets the size of the GLFW window to `width x height`.
    /// Defaults to `640 x 480` if not given.
    pub fn size(mut self, width: u32, height: u32) -> WindowBuilder<'glfw, 'title, 'monitor> {
        self.size = Some((width, height));
        self
    }

    /// Sets the title of the GLFW window to `title`.
    /// Defaults to `"GLFW Window"` if not given.
    pub fn title(mut self, title: &'title str) -> WindowBuilder<'glfw, 'title, 'monitor> {
        self.title = Some(title);
        self
    }

    /// Sets the mode of the GLFW window to `mode`.
    /// Defaults to `Windowed` if not given.
    pub fn mode(mut self, mode: WindowMode<'monitor>) -> WindowBuilder<'glfw, 'title, 'monitor> {
        self.mode = Some(mode);
        self
    }

    /// Tell the OpenGL context that it can expect no errors from your program
    pub fn no_error(self) -> WindowBuilder<'glfw, 'title, 'monitor> {
        self.common_hints(&[
            WindowHint::ContextNoError(true)
        ])
    }

    /// Set the desired refresh rate of the GLFW window. If set to `None`,
    /// it will try for the highest refresh rate possible
    pub fn refresh_rate(self, rate: Option<u32>) -> WindowBuilder<'glfw, 'title, 'monitor> {
        self.common_hints(&[
            WindowHint::RefreshRate(rate)
        ])
    }

    /// Adds a list of `WindowHint`s to try creating a window with.
    ///
    /// If multiple `try_hints()` calls are present, then only one of them
    /// will be applied (the first that lead to a successful window creation).
    ///
    /// This method works in combination with `common_hints()` to create
    /// a list of fallback window configurations to try initializing with.
    /// For details, see `create()`.
    pub fn try_hints(mut self, hints: &[WindowHint]) -> WindowBuilder<'glfw, 'title, 'monitor> {
        self.try_hints.push(hints.iter().map(|&x| x).collect());
        self
    }

    /// Sets the aspect ratio of the window.
    /// It will try to constrain the dimensions to this ratio even when resizing the window
    pub fn aspect_ratio(mut self, numer: u32, denom: u32) -> WindowBuilder<'glfw, 'title, 'monitor> {
        self.aspect_ratio = Some((numer, denom));
        self
    }

    /// Adds a list of `WindowHint`s for the window to be created.
    ///
    /// If multiple `common_hints()` calls are present, they will all be
    /// applied for the created window in the order they where given.
    ///
    /// This method works in combination with `try_hints()` to create
    /// a list of fallback window configurations to try initializing with.
    /// For details, see `create()`.
    pub fn common_hints(mut self, hints: &[WindowHint]) -> WindowBuilder<'glfw, 'title, 'monitor> {
        self.common_hints.extend(hints.iter().map(|&x| x));
        self
    }

    /// Applies a number of `try_hints()` with the goal of getting
    /// a modern OpenGL context version.
    ///
    /// Modern in this context is defined as providing
    /// GLSL support, and providing as many extensions as possible,
    /// ideally approaching version 3.2 or higher.
    ///
    /// Specifically, this adds two `try_hints()` calls that try for 3.2 core and 2.0,
    /// in that order.
    ///
    /// This method exists as a cross-platform compatible way to get a context that
    /// supports newer OpenGL feature under OS X, as 10.7+ supports OpenGL
    /// 3.2 but defaults to a 2.1 context that does _not_ expose the additional
    /// extensions.
    pub fn try_modern_context_hints(self) -> WindowBuilder<'glfw, 'title, 'monitor> {
        // OS X requires forward compatability, annoyingly enough.
        self.try_hints(&[
            WindowHint::ContextVersion(4, 5),
            WindowHint::OpenGlForwardCompat(true),
            WindowHint::OpenGlProfile(OpenGlProfileHint::Core)
        ])
            .try_hints(&[
                WindowHint::ContextVersion(4, 4),
                WindowHint::OpenGlForwardCompat(true),
                WindowHint::OpenGlProfile(OpenGlProfileHint::Core)
            ])
            .try_hints(&[
                WindowHint::ContextVersion(4, 3),
                WindowHint::OpenGlForwardCompat(true),
                WindowHint::OpenGlProfile(OpenGlProfileHint::Core)
            ])
            .try_hints(&[
                WindowHint::ContextVersion(4, 2),
                WindowHint::OpenGlForwardCompat(true),
                WindowHint::OpenGlProfile(OpenGlProfileHint::Core)
            ])
            .try_hints(&[
                WindowHint::ContextVersion(4, 1),
                WindowHint::OpenGlForwardCompat(true),
                WindowHint::OpenGlProfile(OpenGlProfileHint::Core)
            ])
            .try_hints(&[
                WindowHint::ContextVersion(4, 0),
                WindowHint::OpenGlForwardCompat(true),
                WindowHint::OpenGlProfile(OpenGlProfileHint::Core)
            ])
            .try_hints(&[
                WindowHint::ContextVersion(3, 2),
                WindowHint::OpenGlForwardCompat(true),
                WindowHint::OpenGlProfile(OpenGlProfileHint::Core),
            ])
            .try_hints(&[
                WindowHint::ContextVersion(3, 1),
                WindowHint::OpenGlForwardCompat(true),
            ])
            .try_hints(&[
                WindowHint::ContextVersion(3, 1),
            ])
            .try_hints(&[
                WindowHint::ContextVersion(3, 0),
                WindowHint::OpenGlForwardCompat(true),
            ])
            .try_hints(&[
                WindowHint::ContextVersion(3, 0),
            ])
            .try_hints(&[
                WindowHint::ContextVersion(2, 1),
            ])
            .try_hints(&[
                WindowHint::ContextVersion(2, 0),
            ])
    }

    /// Try to create the window.
    ///
    /// This method tries each of the possible window hints given
    /// with `try_hints()` in order, returning the first one that succeeds.
    ///
    /// In order for that to work, it has to disable the `Glfw` error callback,
    /// so you'll have to rebind it afterwards.
    ///
    /// For every set of window hints given with a `try_hints()`, this method
    ///
    /// - Applies the window hints of all `common_hints()` given.
    /// - Applies the window hints of the current `try_hints()`.
    /// - Tries to call `glfw.create_window()` with the given arguments
    ///   (or default values).
    /// - Returns on successful window creation.
    pub fn create(self) -> Option<(Window, Receiver<(f64, WindowEvent)>)> {
        let WindowBuilder { glfw, common_hints, try_hints, size, aspect_ratio, title, mode } = self;

        let (width, height) = size.unwrap_or((640, 480));
        let title = title.unwrap_or("GLFW Window");
        let mode = mode.unwrap_or(WindowMode::Windowed);

        for setup in try_hints.iter() {
            glfw.default_window_hints();

            for hint in common_hints.iter() {
                glfw.window_hint(*hint);
            }

            for hint in setup.iter() {
                glfw.window_hint(*hint);
            }

            if let Some((mut window, events)) = glfw.create_window(width, height, title, mode) {
                info!("Created GLFW window with GL context hints {:?} and {:?}", common_hints, setup);

                if let Some((numer, denom)) = aspect_ratio {
                    window.set_aspect_ratio(numer, denom);
                }

                return Some((window, events));
            } else {
                debug!("Couldn't create a context for hints {:?} and {:?}", common_hints, setup);
            }
        }
        None
    }
}
