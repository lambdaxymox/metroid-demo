use math::{Vector3, Vec4, Mat4, Versor};


pub struct Camera {
    pub near: f32,
    pub far: f32,
    pub fov: f32,
    pub aspect: f32,
    pub proj_mat: Mat4,

    pub cam_speed: f32,
    pub cam_yaw_speed: f32,
    pub cam_pos: Vector3,
    pub fwd: Vec4,
    pub rgt: Vec4,
    pub up: Vec4,

    pub trans_mat_inv: Mat4,
    pub axis: Versor,
    pub rot_mat_inv: Mat4,
    pub view_mat: Mat4,
}

impl Camera {
    pub fn new(
        near: f32, far: f32, fov: f32, aspect: f32, 
        cam_speed: f32, cam_yaw_speed: f32, cam_pos: Vector3,
        fwd: Vec4, rgt: Vec4, up: Vec4, axis: Versor) -> Camera {

        let proj_mat = Mat4::perspective(fov, aspect, near, far);
        let trans_mat_inv = Mat4::identity().translate(&cam_pos);
        let rot_mat_inv = axis.to_mat4();
        let view_mat = rot_mat_inv.inverse() * trans_mat_inv.inverse();

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

