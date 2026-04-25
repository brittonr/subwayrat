## ADDED Requirements

### Requirement: LayerId type

`rat-layers` SHALL provide a `LayerId` type backed by UUID v4 for
globally-unique layer identification suitable for CRDT merging.

#### Scenario: LayerId uniqueness

- **WHEN** two `LayerId::new()` calls are made
- **THEN** the returned IDs are distinct

#### Scenario: LayerId equality from same UUID

- **WHEN** two `LayerId` values wrap the same UUID
- **THEN** they are equal and hash to the same value

### Requirement: Layer type

`rat-layers` SHALL provide a `Layer` struct with id, name, visible (bool),
and locked (bool) fields. New layers default to visible=true, locked=false.

#### Scenario: Default layer state

- **WHEN** `Layer::new("Background")` is called
- **THEN** visible is true, locked is false, id is a fresh UUID, name is "Background"

#### Scenario: Layer with explicit id

- **WHEN** `Layer::with_id(id, "Overlay")` is called
- **THEN** the layer uses the provided id

### Requirement: LayerStack type

`rat-layers` SHALL provide a `LayerStack` that maintains an ordered list of
layers with CRUD operations.

#### Scenario: Default stack has one layer

- **WHEN** `LayerStack::new()` is called
- **THEN** it contains exactly one layer named "Layer 1"

#### Scenario: Add layer

- **WHEN** `add_layer("Layer 2")` is called
- **THEN** a new layer is appended to the top of the stack and its LayerId is returned

#### Scenario: Remove layer preserves minimum

- **WHEN** `remove_layer(id)` is called and the stack has only one layer
- **THEN** an error is returned and the layer remains

#### Scenario: Remove layer with items reassigns

- **WHEN** `remove_layer(id)` is called on a layer that owns items
- **THEN** those items are reassigned to the first remaining layer

#### Scenario: Reorder layers

- **WHEN** `move_layer(id, new_index)` is called
- **THEN** the layer moves to the new position (clamped to valid range)
- **THEN** all other layers shift to accommodate

### Requirement: Layer visibility and lock

Setting a layer invisible SHALL cause `is_visible(layer_id)` to return false.
Setting a layer locked SHALL cause `is_locked(layer_id)` to return true.

#### Scenario: Toggle visibility

- **WHEN** `set_visible(id, false)` is called on a visible layer
- **THEN** `is_visible(id)` returns false

#### Scenario: Toggle lock

- **WHEN** `set_locked(id, true)` is called on an unlocked layer
- **THEN** `is_locked(id)` returns true

### Requirement: Item-layer ownership

The layer stack SHALL track which layer each item belongs to via a
`HashMap<ItemId, LayerId>` where `ItemId` is a generic type parameter.

#### Scenario: Assign item to layer

- **WHEN** `set_item_layer(item_id, layer_id)` is called
- **THEN** `get_item_layer(item_id)` returns `Some(layer_id)`

#### Scenario: Unassigned item

- **WHEN** `get_item_layer(item_id)` is called for an item never assigned
- **THEN** it returns `None`

#### Scenario: Items on deleted layer reassigned

- **WHEN** a layer is removed that owns items
- **THEN** all its items are reassigned to the first layer in the stack

### Requirement: Layer ordering for rendering

The layer stack SHALL provide an iterator over layers in render order
(bottom to top) and a method to query the z-index of a layer.

#### Scenario: Render order iteration

- **WHEN** `layers_bottom_to_top()` is called on a stack with layers [A, B, C]
- **THEN** the iterator yields A, B, C in that order

#### Scenario: Z-index query

- **WHEN** `z_index(layer_id)` is called
- **THEN** it returns the 0-based position in the stack (0 = bottom)

### Requirement: Rename layer

The layer stack SHALL allow callers to rename an existing layer by id.

#### Scenario: Rename updates name

- **WHEN** `rename_layer(id, "New Name")` is called
- **THEN** the layer's name becomes "New Name"

### Requirement: No ratatui dependency in core

The `rat-layers` crate SHALL not depend on `ratatui` in its default feature
set. A `widget` feature MAY gate an optional layer panel widget.

#### Scenario: Core compiles without ratatui

- **WHEN** `rat-layers` is compiled with default features
- **THEN** it does not pull in `ratatui`
