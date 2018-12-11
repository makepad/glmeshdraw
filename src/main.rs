extern crate gl;
extern crate glutin;

use std::ffi::CStr;
use glutin::dpi::*;
use glutin::GlContext;
use std::mem;
use std::ptr;
use std::fs::File; 
use std::io::BufRead;
use std::io::BufReader;

static instance_props: [f32; 2] = [
    0.0, 0.0
];

#[derive(Debug)]
struct Mat4{
    v: [f32; 16],
}

struct Vec3{
    x: f32,
    y: f32,
    z: f32
}

impl Mat4{
    fn identity() -> Mat4{
        return Mat4{
            v:[
                1.0,0.0,0.0,0.0,
                0.0,1.0,0.0,0.0,
                0.0,0.0,1.0,0.0,
                0.0,0.0,0.0,1.0
            ]
        }
    }

    fn rotate_tsrt(t1: Vec3, s: Vec3, r: Vec3, t2: Vec3) -> Mat4{
        let cx = f32::cos(r.x);
        let cy = f32::cos(r.y);
        let cz = f32::cos(r.z);
        let sx = f32::sin(r.x);
        let sy = f32::sin(r.y);
        let sz = f32::sin(r.z);
        let m0 = s.x * (cy * cz + sx * sy * sz);
        let m1 = s.y * (-sz * cy + cz * sx * sy);
        let m2 = s.z * (sy * cx);
        let m4 = s.x * (sz * cx);
        let m5 = s.y * (cx * cz);
        let m6 = s.z * (-sx);
        let m8 = s.x * (-sy * cz + cy * sx * sz);
        let m9 = s.y * (sy * sz + cy * sx * cz);
        let m10 = s.z * (cx * cy);
        return Mat4{
            v:[
                m0, m4, m8, 0.0,
                m1, m5, m9, 0.0,
                m2, m6, m10, 0.0,
                t2.x + (m0 * t1.x + m1 * t1.y + m1 * t1.z),
                t2.y + (m4 * t1.x + m5 * t1.y + m6 * t1.z),
                t2.z + (m8 * t1.x + m9 * t1.y + m10 * t1.z),
                1.0
            ]
        }
    }

    fn perspective(fov_y:f32, aspect:f32, near:f32, far:f32) -> Mat4{
        let f = 1.0 / f32::tan(fov_y / 2.0);
        let nf = 1.0 / (near - far);
        return Mat4{
            v:[
                f / aspect, 0.0, 0.0, 0.0,
                0.0, f , 0.0, 0.0,
                0.0, 0.0, (far + near) * nf, -1.0,
                0.0, 0.0, (2.0 * far * near) * nf, 0.0

            ]
        }
    }
} 

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;

uniform mat4 projection;
uniform mat4 camera;
uniform float mode;

attribute vec3 position;
attribute vec2 disp;
varying vec3 color;

void main() {
    mat4 total = projection * camera;
    gl_Position =  total * vec4(position, 1.);

    color = position*vec3(0.5) + vec3(0.5);
    if(mode>0.5) color = vec3(1.0,1.0,1.0);
}
\0";

const FS_SRC: &[u8] = b"
#version 100
precision mediump float;
varying vec3 color;
void main() {
    // do a screenspace normal

    gl_FragColor = vec4(color,1.0);//vec4(0.1,0.4,0.6,1.0);
}

\0";

fn main() {


    let mut load_vertices = Vec::new();
    let mut load_indices = Vec::new();
    // read OBJ file
    let f = File::open("test.obj").unwrap();
    let mut file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let split: Vec<&str> = l.split(" ").collect();
        let item = split[0];
        if item == "f"{
            load_indices.push(split[1].parse::<i32>().unwrap()-1);
            load_indices.push(split[2].parse::<i32>().unwrap()-1);
            load_indices.push(split[3].parse::<i32>().unwrap()-1);
        }
        else{
            load_vertices.push(split[1].parse::<f32>().unwrap());
            load_vertices.push(split[2].parse::<f32>().unwrap());
            load_vertices.push(split[3].parse::<f32>().unwrap());
        }
    }

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Hello, world")
        .with_dimensions(LogicalSize::new(640.0, 480.0));
    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    
    unsafe {
        gl_window.make_current().unwrap();
    }
    let mut camera_uni = 0;
    let mut mode_uni = 0;
    unsafe {
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.2, 0.2, 0.2, 1.0);

        let vs = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
        gl::CompileShader(vs);
        let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fs, 1, [FS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
        gl::CompileShader(fs);

        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        gl::UseProgram(program);
        
        let mut vb = mem::uninitialized();
        gl::GenBuffers(1, &mut vb);
        gl::BindBuffer(gl::ARRAY_BUFFER, vb);
        gl::BufferData(gl::ARRAY_BUFFER,
                           (load_vertices.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                           load_vertices.as_ptr() as *const _, gl::STATIC_DRAW);

        //if gl::BindVertexArray.is_loaded() {
            let mut vao = mem::uninitialized();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        //}


        let mut ib = mem::uninitialized();
        gl::GenBuffers(1, &mut ib);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (load_indices.len() * mem::size_of::<i32>()) as gl::types::GLsizeiptr,
                           load_indices.as_ptr() as *const _, gl::STATIC_DRAW);


        let pos_attrib = gl::GetAttribLocation(program, b"position\0".as_ptr() as *const _);

        gl::VertexAttribPointer(pos_attrib as gl::types::GLuint, 3, gl::FLOAT, 0,
                                    3 * mem::size_of::<f32>() as gl::types::GLsizei,
                                    ptr::null());

        gl::EnableVertexAttribArray(pos_attrib as gl::types::GLuint);

        let mut vbi = mem::uninitialized();
        gl::GenBuffers(1, &mut vbi);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbi);
        gl::BufferData(gl::ARRAY_BUFFER,
                          (instance_props.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                           instance_props.as_ptr() as *const _, gl::STATIC_DRAW);

        let disp_attrib = gl::GetAttribLocation(program, b"disp\0".as_ptr() as *const _);
        gl::VertexAttribPointer(disp_attrib as gl::types::GLuint, 2, gl::FLOAT, 0,
                                    2 * mem::size_of::<f32>() as gl::types::GLsizei,
                                    ptr::null());

        gl::EnableVertexAttribArray(disp_attrib as gl::types::GLuint);
        gl::VertexAttribDivisor(disp_attrib as gl::types::GLuint, 1 as gl::types::GLuint);
        
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
          
        camera_uni = gl::GetUniformLocation(program, b"camera\0".as_ptr() as *const _);
        mode_uni = gl::GetUniformLocation(program, b"mode\0".as_ptr() as *const _);

         let projection_matrix = Mat4::perspective(1.5, 4.0/3.0, 0.01, 100.0);
        let projection_uni = gl::GetUniformLocation(program, b"projection\0".as_ptr() as *const _);
        gl::UniformMatrix4fv(projection_uni, 1, 0, projection_matrix.v.as_ptr());

    }

    let version = unsafe {
        let data = CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_bytes().to_vec();
        String::from_utf8(data).unwrap()
    };

    println!("OpenGL version {}", version);

    let mut first = true;
    let mut running = true;
    let mut frame = 0.0;

    while running {
        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent{ event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        let dpi_factor = gl_window.get_hidpi_factor();
                        gl_window.resize(logical_size.to_physical(dpi_factor));
                        // lets resize the fractal framebuffer

                    },
                    _ => ()
                },
                _ => ()
            }
        });

        if first == true{
            first = false;
            let new_logical = LogicalSize::new(800.0,600.0);
            gl_window.set_inner_size(new_logical);
        }

        let size = gl_window.get_inner_size().unwrap();
        let dpi_factor = gl_window.get_hidpi_factor();
        let mysize = size.to_physical(dpi_factor);
        frame = frame + 1.0;

        let camera_matrix = Mat4::rotate_tsrt(
            Vec3{x:0.0,y:0.0,z:0.0},
            Vec3{x:1.0,y:1.0,z:1.0},
            Vec3{x:frame/10000.0,y:frame/10000.0,z:0.0},
            Vec3{x:0.0,y:0.0,z:-1.5}
        );

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT|gl::DEPTH_BUFFER_BIT);
            gl::UniformMatrix4fv(camera_uni, 1, 0, camera_matrix.v.as_ptr());

            gl::Enable(gl::POLYGON_OFFSET_FILL);
            gl::PolygonOffset(1.0,100.0);
            gl::Uniform1f(mode_uni, 0.0);
            gl::DrawElementsInstanced(gl::TRIANGLES,load_indices.len() as i32, gl::UNSIGNED_INT, ptr::null(), 1);
            gl::Uniform1f(mode_uni, 1.0);

            gl::Uniform1f(mode_uni, 1.0);
            gl::DrawElementsInstanced(gl::LINE_LOOP,load_indices.len() as i32, gl::UNSIGNED_INT, ptr::null(), 1);

            gl::Uniform1f(mode_uni, 2.0);
            gl::PointSize(10.0);
            gl::DrawElementsInstanced(gl::POINTS,load_indices.len() as i32, gl::UNSIGNED_INT, ptr::null(), 1);

        }

        gl_window.swap_buffers().unwrap();

    }
}