//! Engine world model for fields (Phase 2b): per-tile field lists
//! mirroring the protocol (`FieldOnTile` + `DisplayedField` in
//! `schema/cdda_ipc.fbs`), and the displayed-field composite weighting
//! (gate 2b.4).
//!
//! The game already decides which field dominates a tile
//! (`displayed_field`); the effect pass weights volumes by that
//! decision instead of re-deriving the game's rule.

use std::collections::BTreeMap;

/// One field on one tile: (field type def id, intensity 1..=3).
/// Mirrors the protocol's `FieldOnTile` minus the tile index (the
/// owning tile is the key in the world model).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldEntry {
    pub field_type: u32,
    pub intensity: u8,
}

/// All fields on one tile plus the game's displayed-field choice.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TileFields {
    pub fields: Vec<FieldEntry>,
    /// The game's cheap path: (field_type, intensity); `None` or a
    /// field_type of 0 means "no displayed field".
    pub displayed: Option<FieldEntry>,
}

/// How much more a displayed field weighs than a non-displayed field
/// of the same intensity. With 4.0, even an intensity-1 displayed
/// field outweighs an intensity-3 non-displayed field (4 > 3), which
/// is what "the displayed field dominates the composite" means.
pub const DISPLAYED_WEIGHT: f32 = 4.0;

/// Composite weights for a tile's fields, normalized to sum to 1,
/// sorted by field_type. Pure function (gate 2b.4).
///
/// Rules:
/// - duplicate entries for one type merge at their max intensity;
/// - the displayed field (when it names a real type) weighs
///   `intensity * DISPLAYED_WEIGHT`, others weigh `intensity`;
/// - a displayed field absent from the list is composited on its own
///   (the game says it dominates; there is nothing else to show);
/// - no fields and no displayed field => empty.
pub fn composite_weights(tile: &TileFields) -> Vec<(u32, f32)> {
    let mut by_type: BTreeMap<u32, u8> = BTreeMap::new();
    for f in &tile.fields {
        if f.field_type == 0 || f.intensity == 0 {
            continue;
        }
        let e = by_type.entry(f.field_type).or_insert(0);
        *e = (*e).max(f.intensity);
    }
    let displayed = tile
        .displayed
        .filter(|d| d.field_type != 0 && d.intensity != 0);
    if let Some(d) = displayed {
        if !by_type.contains_key(&d.field_type) && by_type.is_empty() {
            return vec![(d.field_type, 1.0)];
        }
        // If the displayed type is listed, its listed intensity rules;
        // if it is not listed but other fields are, include it at its
        // own intensity so the game's choice is never invisible.
        by_type.entry(d.field_type).or_insert(d.intensity);
    }
    if by_type.is_empty() {
        return Vec::new();
    }
    let disp_type = displayed.map(|d| d.field_type);
    let mut total = 0.0f32;
    let raw: Vec<(u32, f32)> = by_type
        .iter()
        .map(|(t, i)| {
            let w = *i as f32
                * if Some(*t) == disp_type {
                    DISPLAYED_WEIGHT
                } else {
                    1.0
                };
            total += w;
            (*t, w)
        })
        .collect();
    raw.into_iter().map(|(t, w)| (t, w / total)).collect()
}

/// Weight of one type in a composite (0.0 when absent).
pub fn weight_of(weights: &[(u32, f32)], field_type: u32) -> f32 {
    weights
        .iter()
        .find(|(t, _)| *t == field_type)
        .map(|(_, w)| *w)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(fields: &[(u32, u8)], displayed: Option<(u32, u8)>) -> TileFields {
        TileFields {
            fields: fields
                .iter()
                .map(|(t, i)| FieldEntry {
                    field_type: *t,
                    intensity: *i,
                })
                .collect(),
            displayed: displayed.map(|(t, i)| FieldEntry {
                field_type: t,
                intensity: i,
            }),
        }
    }

    #[test]
    fn weights_sum_to_one_and_sort() {
        let w = composite_weights(&tile(&[(7, 2), (3, 1)], None));
        assert_eq!(w.len(), 2);
        assert_eq!(w[0].0, 3);
        assert!((w.iter().map(|(_, x)| x).sum::<f32>() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn empty_tile_has_no_weights() {
        assert!(composite_weights(&TileFields::default()).is_empty());
        assert!(composite_weights(&tile(&[], Some((0, 0)))).is_empty());
    }

    #[test]
    fn displayed_alone_is_full_weight() {
        let w = composite_weights(&tile(&[], Some((5, 2))));
        assert_eq!(w, vec![(5, 1.0)]);
    }
}
