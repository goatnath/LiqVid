use crate::mesh::Mesh;
use nalgebra::Vector3;
#[allow(dead_code)]
#[derive(Clone)]
pub enum BcType<T> {
    //Dirichlet
    FixedValue(T),
    //Neumann
    ZeroGradient,
}

#[derive(Clone)]
pub struct BoundaryField<T> {
    pub x_min: BcType<T>, //left
    pub x_max: BcType<T>, //right
    pub y_min: BcType<T>, //bottom
    pub y_max: BcType<T>, //top
    pub z_min: BcType<T>, //back
    pub z_max: BcType<T>, //front
}

impl<T: Clone> BoundaryField<T> {
    pub fn new_zero_gradient() -> Self {
        Self {
            x_min: BcType::ZeroGradient,
            x_max: BcType::ZeroGradient,
            y_min: BcType::ZeroGradient,
            y_max: BcType::ZeroGradient,
            z_min: BcType::ZeroGradient,
            z_max: BcType::ZeroGradient,
        }
    }
}

pub struct VolScalarField {
    pub internal_field: Vec<f64>,
    pub boundary_field: BoundaryField<f64>,
}
pub struct VolVectorField {
    pub internal_field: Vec<Vector3<f64>>,
    pub boundary_field: BoundaryField<Vector3<f64>>,
}
#[allow(dead_code)]
pub struct SurfaceScalarField {
    pub x_flux: Vec<f64>,
    pub y_flux: Vec<f64>,
    pub z_flux: Vec<f64>,
}

impl VolScalarField {
    pub fn new(mesh: &Mesh, default_value: f64) -> Self {
        Self {
            internal_field: vec![default_value; mesh.num_cells()],
            boundary_field: BoundaryField::new_zero_gradient(),
            //default is zero gradient
        }
    }
}

impl VolVectorField {
    pub fn new(mesh: &Mesh, default_value: Vector3<f64>) -> Self {
        Self {
            internal_field: vec![default_value; mesh.num_cells()],
            boundary_field: BoundaryField::new_zero_gradient(),
            //default is zero gradient
        }
    }
}
#[allow(dead_code)]
impl SurfaceScalarField {
    pub fn new(mesh: &Mesh, default_value: f64) -> Self {
        Self {
            x_flux: vec![default_value; (mesh.nx + 1) * mesh.ny * mesh.nz],
            y_flux: vec![default_value; mesh.nx * (mesh.ny + 1) * mesh.nz],
            z_flux: vec![default_value; mesh.nx * mesh.ny * (mesh.nz + 1)],
        }
    }
}
