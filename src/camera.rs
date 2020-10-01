use cglinalg::{
    Degrees, 
    Vector3, 
    Vector4, 
    Matrix4, 
    Quaternion, 
    InvertibleSquareMatrix
};


pub struct Camera {
    pub near: f32,
    pub far: f32,
    pub fov: f32,
    pub aspect: f32,
    pub proj_mat: Matrix4<f32>,

    pub cam_speed: f32,
    pub cam_yaw_speed: f32,
    pub cam_pos: Vector3<f32>,
    pub fwd: Vector4<f32>,
    pub rgt: Vector4<f32>,
    pub up: Vector4<f32>,

    pub trans_mat_inv: Matrix4<f32>,
    pub axis: Quaternion<f32>,
    pub rot_mat_inv: Matrix4<f32>,
    pub view_mat: Matrix4<f32>,
}

impl Camera {
    pub fn new(
        near: f32, far: f32, fov: f32, aspect: f32, 
        cam_speed: f32, cam_yaw_speed: f32, cam_pos: Vector3<f32>,
        fwd: Vector4<f32>, rgt: Vector4<f32>, up: Vector4<f32>, axis: Quaternion<f32>) -> Camera {

        let proj_mat = Matrix4::from_perspective_fov(Degrees(fov), aspect, near, far);
        let trans_mat_inv = Matrix4::from_affine_translation(&cam_pos);
        let rot_mat_inv = Matrix4::from(axis);
        let view_mat = rot_mat_inv.inverse().unwrap() * trans_mat_inv.inverse().unwrap();

        Camera {
            near: near,
            far: far,
            fov: fov,
            aspect: aspect,
            proj_mat: proj_mat,

            cam_speed: cam_speed,
            cam_yaw_speed: cam_yaw_speed,
            cam_pos: cam_pos,
            fwd: fwd,
            rgt: rgt,
            up: up,

            trans_mat_inv: trans_mat_inv,
            axis: axis,
            rot_mat_inv: rot_mat_inv,
            view_mat: view_mat,
        }
    }
}

