//! Bake-side textured occlusion gate
//! (docs/fine-slope-normals.md §14): the fine mesher's
//! Stage H occlusion factor for textured faces rides in
//! `Mesh::tex_occlusion`, NOT in the vertex colour — the
//! bake replaces a textured vertex's RGB with the pure
//! light multiplier, and the raster shader reads that RGB
//! only as the emitter ceiling, so a factor folded into
//! either place would never reach the diffuse term (and
//! would double-apply once the shader applies the factor
//! from its own vertex attribute). The gate pins the
//! bake's side of that contract, against one uniform
//! BRIGHT grid:
//!
//! - a textured vertex's baked RGB is the pure light
//!   multiplier whether or not factors are recorded —
//!   the recorded factors must NOT leak into it;
//! - `tex_occlusion` passes through the bake byte-for-byte
//!   (the engine uploads it as a vertex attribute after
//!   the bake; losing or rewriting it here would silently
//!   drop the occlusion).

use lighting::grid::LightGrid;
use lighting::mapping::TileLight;
use lighting::{bake_pass_meshes, Environment};
use mesher::mesh::{Mesh, PassMeshes};
use mesher::scale::WorldScale;
use proto::LitLevel;

fn textured_quad(factors: Option<[f32; 4]>) -> PassMeshes {
    let bs = WorldScale::default().block_size;
    let corners = [
        [0.0, bs, 0.0],
        [bs, bs, 0.0],
        [bs, bs, -bs],
        [0.0, bs, -bs],
    ];
    let normals = [[0.0, 1.0, 0.0]; 4];
    let colors = [[0.9, 0.9, 0.9, 1.0]; 4];
    let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let mut m = Mesh::default();
    match factors {
        Some(f) => {
            m.push_quad_textured_per_corner_colors_occ(corners, normals, colors, f, uvs, 7)
        }
        None => m.push_quad_textured_per_corner_colors(corners, normals, colors, uvs, 7),
    }
    let mut pm = PassMeshes::default();
    pm.meshes[0] = m;
    pm
}

fn bake(mut pm: PassMeshes) -> PassMeshes {
    let g = LightGrid::filled(
        [0, 0, 0],
        [4, 4, 1],
        TileLight::new(LitLevel::BRIGHT, 255),
    );
    bake_pass_meshes(&mut pm, &g, Environment::neutral(), WorldScale::default(), 0);
    pm
}

#[test]
fn bake_keeps_the_multiplier_pure_and_the_factors_untouched() {
    let f = [0.5, 0.25, 1.0, 0.75];
    let plain = bake(textured_quad(None));
    assert!(
        plain.meshes[0].colors[0][0] > 0.05,
        "fixture must produce a real multiplier, got {:?}",
        plain.meshes[0].colors[0]
    );
    let with = bake(textured_quad(Some(f)));
    assert_eq!(
        with.meshes[0].colors, plain.meshes[0].colors,
        "recorded factors must not leak into the baked multiplier"
    );
    assert_eq!(
        with.meshes[0].tex_occlusion,
        f.to_vec(),
        "the factor array must pass through the bake verbatim"
    );
    assert!(
        plain.meshes[0].tex_occlusion.is_empty(),
        "no factors recorded, none invented"
    );
}
