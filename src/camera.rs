use wgpu::util::DeviceExt;
use cgmath::{
    Angle,
    InnerSpace
};
use cgmath::{
    Matrix4,
    Vector3,
    Point3,
    Rad,
    Deg
};
use sdl2::{
    event::Event,
    keyboard::Keycode
};
use std::time::Duration;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraProjectionRaw {
    position: [f32; 4],
    proj_matrix: [[f32; 4]; 4]
}

impl CameraProjectionRaw {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 0.0],
            proj_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]
        }
    }

    pub fn update_proj_matrix(&mut self, camera_proj: &CameraProjection, camera: &Camera) {
        self.position = camera.position.to_homogeneous().into();
        self.proj_matrix = camera_proj.build_proj_matrix(camera).into();
    }
}

pub struct CameraProjection {
    fovy: Rad<f32>,
    aspect: f32,
    near: f32,
    far: f32,
    up: Vector3<f32>
}

impl CameraProjection {
    const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.5,
        0.0, 0.0, 0.0, 1.0
    );

    pub fn new<F>(fovy: F, container_width: f32, container_height: f32, near: f32, far: f32) -> Self
    where
        F: Into<Rad<f32>>,
    {
        Self {
            fovy: fovy.into(),
            aspect: container_width / container_height,
            near,
            far,
            up: Vector3::unit_y()
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    fn build_proj_matrix(&self, camera: &Camera) -> Matrix4<f32> {
        Self::OPENGL_TO_WGPU_MATRIX *
        cgmath::perspective(self.fovy, self.aspect, self.near, self.far) *
        Matrix4::look_to_rh(camera.position, camera.calc_dir_vector(), self.up)
    }
}

pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>
}

impl Camera {
    pub fn new<V, Y, P>(position: V, yaw: Y, pitch: P) -> Self
    where
        V: Into<Point3<f32>>,
        Y: Into<Rad<f32>>,
        P: Into<Rad<f32>>
    {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into()
        }
    }

    fn calc_dir_vector(&self) -> Vector3<f32> {
        Vector3::new(self.yaw.cos(), self.pitch.tan(), self.yaw.sin())
        // Vector3::new(self.yaw.cos() * self.pitch.cos(), self.pitch.sin(), self.yaw.sin() * self.pitch.cos())
    }

    pub fn dirs_forward_right(&self) -> (Vector3<f32>, Vector3<f32>) {
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        (Vector3::new(yaw_cos, 0.0, yaw_sin),
        Vector3::new(-yaw_sin, 0.0, yaw_cos))
    }
}

pub struct CameraController {
    delta_forward: f32,
    delta_right: f32,
    delta_up: f32,
    delta_yaw: f32,
    delta_pitch: f32,
    move_speed: f32,
    rot_speed: f32
}

impl CameraController {
    pub fn new(move_speed: f32, rot_speed: f32) -> Self {
        Self {
            delta_forward: 0.0,
            delta_right: 0.0,
            delta_up: 0.0,
            delta_yaw: 0.0,
            delta_pitch: 0.0,
            move_speed,
            rot_speed
        }
    }

    pub fn process_event(&mut self, event: Event) {
        match event {
            Event::KeyDown { keycode: Some(keycode), .. } => match keycode {
                Keycode::W => { self.delta_forward = self.move_speed },
                Keycode::S => { self.delta_forward = -self.move_speed },
                Keycode::D => { self.delta_right = self.move_speed },
                Keycode::A => { self.delta_right = -self.move_speed },

                Keycode::Space => { self.delta_up = self.move_speed },
                Keycode::LShift | Keycode::RShift => { self.delta_up = -self.move_speed },

                Keycode::Right => { self.delta_yaw = self.rot_speed },
                Keycode::Left => { self.delta_yaw = -self.rot_speed },
                Keycode::Up => { self.delta_pitch = self.rot_speed },
                Keycode::Down => { self.delta_pitch = -self.rot_speed },


                _ => {}
            },

            Event::KeyUp { keycode: Some(keycode), .. } => match keycode {
                Keycode::W | Keycode::S => { self.delta_forward = 0.0 },
                Keycode::D | Keycode::A => { self.delta_right = 0.0 },

                Keycode::Space |
                Keycode::LShift | Keycode::RShift => { self.delta_up = 0.0 },

                Keycode::Right | Keycode::Left => { self.delta_yaw = 0.0 },
                Keycode::Up | Keycode::Down => { self.delta_pitch = 0.0 },

                _ => {}
            }

            _ => {}
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, deltatime: &Duration) {
        let (forward, right) = camera.dirs_forward_right();
        let forward = forward * self.move_speed;
        let right = right * self.move_speed;
        let deltatime = deltatime.as_secs_f32();

        camera.position += (self.delta_forward * forward + self.delta_right * right + self.delta_up * Vector3::unit_y()) * deltatime;

        camera.yaw.0 += self.delta_yaw * deltatime;
        camera.pitch.0 += self.delta_pitch * deltatime;
        // Change to use Rust's PI constants
        if camera.pitch > Rad::from(Deg(90.0)) {
            camera.pitch = Rad::from(Deg(90.0 - 0.0001));
        }
        if camera.pitch < Rad::from(Deg(-90.0)) {
            camera.pitch = Rad::from(Deg(-90.0 + 0.0001));
        }
    }
}
