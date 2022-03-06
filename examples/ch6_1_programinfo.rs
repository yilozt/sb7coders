// Copyright � 2012-2015 Graham Sellers
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice (including the next
// paragraph) shall be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use sb7::prelude::{AppConfig, Application};

fn type_to_name(t: u32) -> &'static str {
    match t {
        gl::FLOAT => "float",
        gl::FLOAT_VEC2 => "vec2",
        gl::FLOAT_VEC3 => "vec3",
        gl::FLOAT_VEC4 => "vec4",
        gl::DOUBLE => "double",
        gl::DOUBLE_VEC2 => "dvec2",
        gl::DOUBLE_VEC3 => "dvec3",
        gl::DOUBLE_VEC4 => "dvec4",
        gl::INT => "int",
        gl::INT_VEC2 => "ivec2",
        gl::INT_VEC3 => "ivec3",
        gl::INT_VEC4 => "ivec4",
        gl::UNSIGNED_INT => "uint",
        gl::UNSIGNED_INT_VEC2 => "uvec2",
        gl::UNSIGNED_INT_VEC3 => "uvec3",
        gl::UNSIGNED_INT_VEC4 => "uvec4",
        gl::BOOL => "bool",
        gl::BOOL_VEC2 => "bvec2",
        gl::BOOL_VEC3 => "bvec3",
        gl::BOOL_VEC4 => "bvec4",
        _ => "unknown_type",
    }
}

#[inline(always)]
fn parse_str(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes)
        .unwrap_or("!! Invaid UTF-8 string !!")
        .trim_matches('\u{0}')
}

#[derive(Default)]
struct App {
    log: String,
}

impl Application for App {
    fn init(&self) -> sb7::prelude::AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Program Information".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let fs_src = "
            #version 460 core
            
            out vec4 color;
            layout (location = 2) out ivec2 data;
            out float extra;
            
            // in BLOCK0
            // {
            //     vec2 tc;
            //     vec4 color;
            //     flat int foo;
            // } fs_in0;
            
            // in BLOCK1
            // {
            //     vec3 normal[4];
            //     flat ivec3 layers;
            //     double bar;
            // } fs_in1;
            
            void main(void)
            {
                // float val = abs(fs_in0.tc.x + fs_in0.tc.y) * 20.0f;
                // color = vec4(fract(val) >= 0.5 ? 1.0 : 0.25) + fs_in1.normal[3].xyzy;
                color = vec4(1.0);
                data = ivec2(1, 2);
                extra = 9.0;
            }";

        unsafe {
            let fs_src = std::ffi::CString::new(fs_src).unwrap();
            let program = gl::CreateShaderProgramv(gl::FRAGMENT_SHADER, 1, &fs_src.as_ptr());
            // gl::ProgramParameteri(program, gl::PROGRAM_SEPARABLE, gl::TRUE as _);

            // let fs = gl::CreateShader(gl::FRAGMENT_SHADER);

            // let fs_src = std::ffi::CString::new(fs_src).unwrap();
            // gl::ShaderSource(fs, 1, &fs_src.as_ptr(), std::ptr::null());
            // gl::CompileShader(fs);

            // gl::AttachShader(program, fs);
            // gl::LinkProgram(program);

            let mut outputs = 0;

            gl::GetProgramInterfaceiv(
                program,
                gl::PROGRAM_OUTPUT,
                gl::ACTIVE_RESOURCES,
                &mut outputs,
            );

            let props = [gl::TYPE, gl::LOCATION, gl::ARRAY_SIZE];
            let mut params = [0; 4];
            let mut name = [0u8; 64];
            let mut buffer = [0u8; 1024];

            gl::GetProgramInfoLog(program, 1, std::ptr::null_mut(), buffer.as_mut_ptr() as _);

            self.log += "Program Linked\n";
            self.log += parse_str(&buffer);

            for i in 0..outputs {
                gl::GetProgramResourceName(
                    program,
                    gl::PROGRAM_OUTPUT,
                    i as _,
                    1024,
                    std::ptr::null_mut(),
                    name.as_mut_ptr() as _,
                );
                gl::GetProgramResourceiv(
                    program,
                    gl::PROGRAM_OUTPUT,
                    i as _,
                    3,
                    props.as_ptr(),
                    3,
                    std::ptr::null_mut(),
                    params.as_mut_ptr(),
                );
                let type_name = type_to_name(params[0] as _);
                if params[2] != 0 {
                    self.log += &format!(
                        "Index {}: {} {}[{}] @ location {}.\n",
                        i,
                        type_name,
                        parse_str(&name),
                        params[2],
                        params[1]
                    );
                } else {
                    self.log += &format!(
                        "Index {}: {} {} @ location {}.\n",
                        i,
                        type_name,
                        parse_str(&name),
                        params[1]
                    );
                }
            }
        }
    }

    fn ui(&mut self, ui: &imgui_glfw_rs::imgui::Ui) {
        use imgui_glfw_rs::imgui;
        let win = imgui::Window::new("OpenGL SuperBible - Program Information")
            .no_decoration()
            .position([10.0, 10.0], imgui::Condition::Always)
            .resizable(false);
        if let Some(end) = win.begin(ui) {
            ui.text(&self.log);
            end.end();
        }
    }
}

fn main() {
    App::default().run();
}
