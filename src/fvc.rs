use crate::fields::{BcType, VolScalarField, VolVectorField};
use crate::geometry::Geometry;
use crate::mesh::Mesh;
use nalgebra::Vector3;

fn scalar_ghost(field: &VolScalarField, center: f64, axis: usize, positive: bool) -> f64 {
    let bc = match (axis, positive) {
        (0, false) => &field.boundary_field.x_min, (0, true) => &field.boundary_field.x_max,
        (1, false) => &field.boundary_field.y_min, (1, true) => &field.boundary_field.y_max,
        (2, false) => &field.boundary_field.z_min, _ => &field.boundary_field.z_max,
    };
    match bc { BcType::FixedValue(v) => 2.0 * *v - center, BcType::ZeroGradient => center }
}
fn vector_ghost(field: &VolVectorField, center: Vector3<f64>, axis: usize, positive: bool) -> Vector3<f64> {
    let bc = match (axis, positive) {
        (0, false) => &field.boundary_field.x_min, (0, true) => &field.boundary_field.x_max,
        (1, false) => &field.boundary_field.y_min, (1, true) => &field.boundary_field.y_max,
        (2, false) => &field.boundary_field.z_min, _ => &field.boundary_field.z_max,
    };
    match bc { BcType::FixedValue(v) => *v * 2.0 - center, BcType::ZeroGradient => center }
}
fn scalar_neighbor(field: &VolScalarField, mesh: &Mesh, geom: &Geometry, i: usize, j: usize, k: usize, axis: usize, positive: bool, center: f64) -> f64 {
    let (ni,nj,nk,inside) = match (axis,positive) {
        (0,true)=>(i+1,j,k,i+1<mesh.nx),(0,false)=>(i.saturating_sub(1),j,k,i>0),
        (1,true)=>(i,j+1,k,j+1<mesh.ny),(1,false)=>(i,j.saturating_sub(1),k,j>0),
        (2,true)=>(i,j,k+1,k+1<mesh.nz),(2,false)=>(i,j,k.saturating_sub(1),k>0),
        _ => unreachable!(),
    };
    if inside { let idx=mesh.cell_idx(ni,nj,nk); if geom.is_solid[idx] { center } else { field.internal_field[idx] } }
    else { scalar_ghost(field,center,axis,positive) }
}
fn vector_neighbor(field: &VolVectorField, mesh: &Mesh, geom: &Geometry, i: usize, j: usize, k: usize, axis: usize, positive: bool, center: Vector3<f64>) -> Vector3<f64> {
    let (ni,nj,nk,inside) = match (axis,positive) {
        (0,true)=>(i+1,j,k,i+1<mesh.nx),(0,false)=>(i.saturating_sub(1),j,k,i>0),
        (1,true)=>(i,j+1,k,j+1<mesh.ny),(1,false)=>(i,j.saturating_sub(1),k,j>0),
        (2,true)=>(i,j,k+1,k+1<mesh.nz),(2,false)=>(i,j,k.saturating_sub(1),k>0),
        _ => unreachable!(),
    };
    if inside { let idx=mesh.cell_idx(ni,nj,nk); if geom.is_solid[idx] { Vector3::zeros() } else { field.internal_field[idx] } }
    else { vector_ghost(field,center,axis,positive) }
}

pub fn grad(field: &VolScalarField, mesh: &Mesh, geom: &Geometry) -> VolVectorField {
    let mut out=VolVectorField::new(mesh,Vector3::zeros());
    for i in 0..mesh.nx { for j in 0..mesh.ny { for k in 0..mesh.nz {
        let idx=mesh.cell_idx(i,j,k); if geom.is_solid[idx] { continue; }
        let c=field.internal_field[idx];
        let dx=(scalar_neighbor(field,mesh,geom,i,j,k,0,true,c)-scalar_neighbor(field,mesh,geom,i,j,k,0,false,c))/(2.0*mesh.dx);
        let dy=(scalar_neighbor(field,mesh,geom,i,j,k,1,true,c)-scalar_neighbor(field,mesh,geom,i,j,k,1,false,c))/(2.0*mesh.dy);
        let dz=(scalar_neighbor(field,mesh,geom,i,j,k,2,true,c)-scalar_neighbor(field,mesh,geom,i,j,k,2,false,c))/(2.0*mesh.dz);
        out.internal_field[idx]=Vector3::new(dx,dy,dz);
    }}}
    out
}
pub fn div(field: &VolVectorField, mesh: &Mesh, geom: &Geometry) -> VolScalarField {
    let mut out=VolScalarField::new(mesh,0.0);
    for i in 0..mesh.nx { for j in 0..mesh.ny { for k in 0..mesh.nz {
        let idx=mesh.cell_idx(i,j,k); if geom.is_solid[idx] { continue; }
        let c=field.internal_field[idx];
        let e=vector_neighbor(field,mesh,geom,i,j,k,0,true,c); let w=vector_neighbor(field,mesh,geom,i,j,k,0,false,c);
        let n=vector_neighbor(field,mesh,geom,i,j,k,1,true,c); let s=vector_neighbor(field,mesh,geom,i,j,k,1,false,c);
        let f=vector_neighbor(field,mesh,geom,i,j,k,2,true,c); let b=vector_neighbor(field,mesh,geom,i,j,k,2,false,c);
        out.internal_field[idx]=(e.x-w.x)/(2.0*mesh.dx)+(n.y-s.y)/(2.0*mesh.dy)+(f.z-b.z)/(2.0*mesh.dz);
    }}}
    out
}
pub fn laplacian_vector(field: &VolVectorField, mesh: &Mesh, geom: &Geometry) -> VolVectorField {
    let mut out=VolVectorField::new(mesh,Vector3::zeros());
    for i in 0..mesh.nx { for j in 0..mesh.ny { for k in 0..mesh.nz {
        let idx=mesh.cell_idx(i,j,k); if geom.is_solid[idx] { continue; }
        let c=field.internal_field[idx];
        let e=vector_neighbor(field,mesh,geom,i,j,k,0,true,c); let w=vector_neighbor(field,mesh,geom,i,j,k,0,false,c);
        let n=vector_neighbor(field,mesh,geom,i,j,k,1,true,c); let s=vector_neighbor(field,mesh,geom,i,j,k,1,false,c);
        let f=vector_neighbor(field,mesh,geom,i,j,k,2,true,c); let b=vector_neighbor(field,mesh,geom,i,j,k,2,false,c);
        out.internal_field[idx]=(e-2.0*c+w)/(mesh.dx*mesh.dx)+(n-2.0*c+s)/(mesh.dy*mesh.dy)+(f-2.0*c+b)/(mesh.dz*mesh.dz);
    }}}
    out
}
pub fn convect(field: &VolVectorField, mesh: &Mesh, geom: &Geometry) -> VolVectorField {
    let mut out=VolVectorField::new(mesh,Vector3::zeros());
    for i in 0..mesh.nx { for j in 0..mesh.ny { for k in 0..mesh.nz {
        let idx=mesh.cell_idx(i,j,k); if geom.is_solid[idx] { continue; }
        let c=field.internal_field[idx];
        let e=vector_neighbor(field,mesh,geom,i,j,k,0,true,c); let w=vector_neighbor(field,mesh,geom,i,j,k,0,false,c);
        let n=vector_neighbor(field,mesh,geom,i,j,k,1,true,c); let s=vector_neighbor(field,mesh,geom,i,j,k,1,false,c);
        let f=vector_neighbor(field,mesh,geom,i,j,k,2,true,c); let b=vector_neighbor(field,mesh,geom,i,j,k,2,false,c);
        let dx=if c.x>=0.0 {(c-w)/mesh.dx} else {(e-c)/mesh.dx};
        let dy=if c.y>=0.0 {(c-s)/mesh.dy} else {(n-c)/mesh.dy};
        let dz=if c.z>=0.0 {(c-b)/mesh.dz} else {(f-c)/mesh.dz};
        out.internal_field[idx]=dx*c.x+dy*c.y+dz*c.z;
    }}}
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::BcType;
    #[test]
    fn fixed_value_boundary_affects_gradient_and_laplacian() {
        let mesh=Mesh::new(3,3,3,3.0,3.0,3.0); let geom=Geometry::new(&mesh);
        let mut p=VolScalarField::new(&mesh,1.0);
        p.boundary_field.x_max=BcType::FixedValue(3.0);
        let g=grad(&p,&mesh,&geom);
        assert!(g.internal_field[mesh.cell_idx(2,1,1)].x > 0.0);
        let mut u=VolVectorField::new(&mesh,Vector3::zeros());
        u.boundary_field.x_max=BcType::FixedValue(Vector3::new(2.0,0.0,0.0));
        let lap=laplacian_vector(&u,&mesh,&geom);
        assert!(lap.internal_field[mesh.cell_idx(2,1,1)].x > 0.0);
    }
}
