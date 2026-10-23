use base64::{Engine as _, engine::general_purpose::STANDARD};
use nalgebra::Vector3;
use serde::Deserialize;

use crate::{
    fields::{BcType, BoundaryField, VolScalarField, VolVectorField},
    geometry::Geometry,
    mesh::Mesh,
};

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Velocity {
    fn vector(self) -> Vector3<f64> {
        Vector3::new(self.x, self.y, self.z)
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Boundary {
    Inlet { velocity: Velocity },
    Outlet { pressure: f64 },
    NoSlip,
    Slip,
    MovingWall { velocity: Velocity },
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Boundaries {
    pub x_min: Boundary,
    pub x_max: Boundary,
    pub y_min: Boundary,
    pub y_max: Boundary,
    pub z_min: Boundary,
    pub z_max: Boundary,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum GeometryConfig {
    Box,
    Cavity,
    ChannelCylinder {
        radius: f64,
        center_x: f64,
        center_y: f64,
    },
    Step {
        step_length: f64,
        step_height: f64,
    },
    BoxObstacle {
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
        min_z: f64,
        max_z: f64,
    },
    Stl {
        stl_base64: String,
    },
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SimulationRequest {
    #[serde(rename = "kinematicViscosity")]
    pub kinematic_viscosity: f64,
    pub density: f64,
    pub geometry: GeometryConfig,
    pub boundaries: Boundaries,
}

impl SimulationRequest {
    pub fn mesh(&self) -> Mesh {
        Mesh::new(50, 50, 30, 100.0, 100.0, 60.0)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.density.is_finite() || self.density <= 0.0 {
            return Err("density must be finite and positive".into());
        }
        if !self.kinematic_viscosity.is_finite() || self.kinematic_viscosity < 0.0 {
            return Err("kinematicViscosity must be finite and nonnegative".into());
        }
        for (axis, bc) in self.boundaries.faces().into_iter().enumerate() {
            match bc {
                Boundary::Inlet { velocity } | Boundary::MovingWall { velocity } => {
                    let v = velocity.vector();
                    if !v.iter().all(|x| x.is_finite()) {
                        return Err("velocity must be finite".into());
                    }
                    if matches!(bc, Boundary::MovingWall { .. }) && v[axis / 2] != 0.0 {
                        return Err("moving-wall velocity must be tangential to its face".into());
                    }
                }
                Boundary::Outlet { pressure } if !pressure.is_finite() => {
                    return Err("pressure must be finite".into());
                }
                _ => {}
            }
        }
        let inside = |v: f64, limit: f64| v.is_finite() && v >= 0.0 && v <= limit;
        let valid = match &self.geometry {
            GeometryConfig::ChannelCylinder {
                radius,
                center_x,
                center_y,
            } => {
                radius.is_finite()
                    && *radius > 0.0
                    && inside(center_x - radius, 100.0)
                    && inside(center_x + radius, 100.0)
                    && inside(center_y - radius, 100.0)
                    && inside(center_y + radius, 100.0)
            }
            GeometryConfig::Step {
                step_length,
                step_height,
            } => {
                *step_length > 0.0
                    && *step_height > 0.0
                    && *step_length < 100.0
                    && *step_height < 100.0
            }
            GeometryConfig::BoxObstacle {
                min_x,
                max_x,
                min_y,
                max_y,
                min_z,
                max_z,
            } => {
                inside(*min_x, 100.0)
                    && inside(*max_x, 100.0)
                    && min_x < max_x
                    && inside(*min_y, 100.0)
                    && inside(*max_y, 100.0)
                    && min_y < max_y
                    && inside(*min_z, 60.0)
                    && inside(*max_z, 60.0)
                    && min_z < max_z
            }
            _ => true,
        };
        if !valid {
            return Err(
                "geometry dimensions must be positive and fit inside the 100 × 100 × 60 domain"
                    .into(),
            );
        }
        Ok(())
    }

    pub fn build_geometry(&self, mesh: &Mesh) -> Result<Geometry, String> {
        let mut geom = Geometry::new(mesh);
        if let GeometryConfig::Stl { stl_base64 } = &self.geometry {
            let bytes = STANDARD
                .decode(stl_base64)
                .map_err(|_| "invalid STL base64".to_string())?;
            geom.load_stl_from_bytes(mesh, &bytes)
                .map_err(|e| format!("invalid STL: {e}"))?;
        } else {
            for k in 0..mesh.nz {
                for j in 0..mesh.ny {
                    for i in 0..mesh.nx {
                        let x = (i as f64 + 0.5) * mesh.dx;
                        let y = (j as f64 + 0.5) * mesh.dy;
                        let z = (k as f64 + 0.5) * mesh.dz;
                        geom.is_solid[mesh.cell_idx(i, j, k)] = match self.geometry {
                            GeometryConfig::ChannelCylinder {
                                radius,
                                center_x,
                                center_y,
                            } => (x - center_x).powi(2) + (y - center_y).powi(2) <= radius.powi(2),
                            GeometryConfig::Step {
                                step_length,
                                step_height,
                            } => x <= step_length && y <= step_height,
                            GeometryConfig::BoxObstacle {
                                min_x,
                                max_x,
                                min_y,
                                max_y,
                                min_z,
                                max_z,
                            } => {
                                x >= min_x
                                    && x <= max_x
                                    && y >= min_y
                                    && y <= max_y
                                    && z >= min_z
                                    && z <= max_z
                            }
                            _ => false,
                        };
                    }
                }
            }
        }
        if geom.is_solid.iter().all(|s| *s) {
            return Err("geometry leaves no fluid cells".into());
        }
        Ok(geom)
    }
}

impl Boundaries {
    fn faces(&self) -> [&Boundary; 6] {
        [
            &self.x_min,
            &self.x_max,
            &self.y_min,
            &self.y_max,
            &self.z_min,
            &self.z_max,
        ]
    }

    pub fn apply(&self, u: &mut VolVectorField, p: &mut VolScalarField) {
        let velocity = self.faces().map(|bc| match bc {
            Boundary::Inlet { velocity } | Boundary::MovingWall { velocity } => {
                BcType::FixedValue(velocity.vector())
            }
            Boundary::NoSlip => BcType::FixedValue(Vector3::zeros()),
            Boundary::Slip => BcType::Slip,
            Boundary::Outlet { .. } => BcType::ZeroGradient,
        });
        let pressure = self.faces().map(|bc| match bc {
            Boundary::Outlet { pressure } => BcType::FixedValue(*pressure),
            _ => BcType::ZeroGradient,
        });
        let [x_min, x_max, y_min, y_max, z_min, z_max] = velocity;
        u.boundary_field = BoundaryField {
            x_min,
            x_max,
            y_min,
            y_max,
            z_min,
            z_max,
        };
        let [x_min, x_max, y_min, y_max, z_min, z_max] = pressure;
        p.boundary_field = BoundaryField {
            x_min,
            x_max,
            y_min,
            y_max,
            z_min,
            z_max,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    pub fn request(geometry: serde_json::Value) -> SimulationRequest {
        serde_json::from_value(json!({
            "kinematicViscosity": 0.01, "density": 1.0, "geometry": geometry,
            "boundaries": {
                "x_min": {"type": "inlet", "velocity": {"x": 2.0, "y": 0.0, "z": 0.0}},
                "x_max": {"type": "outlet", "pressure": 3.0},
                "y_min": {"type": "no_slip"},
                "y_max": {"type": "moving_wall", "velocity": {"x": 1.0, "y": 0.0, "z": 0.0}},
                "z_min": {"type": "slip"}, "z_max": {"type": "slip"}
            }
        }))
        .unwrap()
    }

    #[test]
    fn frontend_contract_maps_all_boundary_types_and_domain() {
        let request = request(json!({"type": "cavity"}));
        request.validate().unwrap();
        let mesh = request.mesh();
        assert_eq!((mesh.nx, mesh.ny, mesh.nz), (50, 50, 30));
        assert_eq!((mesh.lx, mesh.ly, mesh.lz), (100.0, 100.0, 60.0));
        let mut u = VolVectorField::new(&mesh, Vector3::zeros());
        let mut p = VolScalarField::new(&mesh, 0.0);
        request.boundaries.apply(&mut u, &mut p);
        assert!(matches!(u.boundary_field.x_min, BcType::FixedValue(v) if v.x == 2.0));
        assert!(matches!(u.boundary_field.x_max, BcType::ZeroGradient));
        assert!(matches!(p.boundary_field.x_max, BcType::FixedValue(3.0)));
        assert!(matches!(u.boundary_field.y_min, BcType::FixedValue(v) if v == Vector3::zeros()));
        assert!(matches!(u.boundary_field.y_max, BcType::FixedValue(v) if v.x == 1.0));
        assert!(matches!(u.boundary_field.z_min, BcType::Slip));
    }

    #[test]
    fn presets_voxelize_in_domain_coordinates() {
        let cases = [
            (
                json!({"type": "channel_cylinder", "radius": 12.0, "center_x": 35.0, "center_y": 50.0}),
                (17, 24, 15),
            ),
            (
                json!({"type": "step", "step_length": 30.0, "step_height": 40.0}),
                (5, 5, 15),
            ),
            (
                json!({"type": "box_obstacle", "min_x": 35.0, "max_x": 65.0, "min_y": 35.0, "max_y": 65.0, "min_z": 10.0, "max_z": 50.0}),
                (24, 24, 15),
            ),
        ];
        for (geometry, (i, j, k)) in cases {
            let request = request(geometry);
            request.validate().unwrap();
            let mesh = request.mesh();
            let geom = request.build_geometry(&mesh).unwrap();
            assert!(geom.is_solid[mesh.cell_idx(i, j, k)]);
            assert!(!geom.is_solid[mesh.cell_idx(49, 49, 29)]);
        }
        for name in ["box", "cavity"] {
            let request = request(json!({"type": name}));
            assert!(
                !request
                    .build_geometry(&request.mesh())
                    .unwrap()
                    .is_solid
                    .iter()
                    .any(|s| *s)
            );
        }
    }

    #[test]
    fn invalid_parameters_and_stl_are_rejected() {
        let mut request = request(json!({"type": "box"}));
        request.density = 0.0;
        assert!(request.validate().is_err());
        request.density = 1.0;
        request.kinematic_viscosity = -0.1;
        assert!(request.validate().is_err());
        request.kinematic_viscosity = 0.01;
        request.geometry = GeometryConfig::ChannelCylinder {
            radius: -2.0,
            center_x: 35.0,
            center_y: 50.0,
        };
        assert!(request.validate().is_err());
        for data in ["!not-base64", "YWJj"] {
            request.geometry = GeometryConfig::Stl {
                stl_base64: data.into(),
            };
            assert!(request.build_geometry(&request.mesh()).is_err());
        }
        assert!(serde_json::from_value::<GeometryConfig>(json!({"type": "unknown"})).is_err());
    }
}
