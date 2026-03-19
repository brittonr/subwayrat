//! Ordered layer stack with visibility, locking, and item ownership.
//!
//! Provides a generic layer management system with:
//! - Layer ordering (bottom to top)
//! - Visibility and lock controls
//! - Item-to-layer ownership mapping
//! - UUID-based layer identifiers for CRDT compatibility

use std::collections::HashMap;
use std::fmt::{self, Display};
use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Layer identifier backed by UUID v4 for globally-unique identification
/// suitable for CRDT merging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LayerId(pub Uuid);

impl LayerId {
    /// Create a new LayerId with a fresh UUID v4.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LayerId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for LayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Layer data structure with id, name, visibility, and lock state.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
}

impl Layer {
    /// Create a new layer with a fresh id, given name, visible=true, locked=false.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            visible: true,
            locked: false,
        }
    }

    /// Create a new layer with the specified id, given name, visible=true, locked=false.
    pub fn with_id(id: LayerId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            visible: true,
            locked: false,
        }
    }
}

/// Ordered layer stack with item ownership tracking.
///
/// Generic over `I: Eq + Hash + Copy + std::fmt::Debug` which is the item identifier type.
/// Items can be assigned to layers and are tracked via a HashMap.
/// Layers are ordered bottom to top for rendering.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LayerStack<I>
where
    I: Eq + std::hash::Hash + Copy + std::fmt::Debug,
{
    layers: Vec<Layer>,
    item_layers: HashMap<I, LayerId>,
}

impl<I> LayerStack<I>
where
    I: Eq + std::hash::Hash + Copy + std::fmt::Debug,
{
    /// Create a new layer stack with one default layer "Layer 1".
    pub fn new() -> Self {
        Self {
            layers: vec![Layer::new("Layer 1")],
            item_layers: HashMap::new(),
        }
    }

    /// Add a new layer to the top of the stack and return its id.
    pub fn add_layer(&mut self, name: &str) -> LayerId {
        let layer = Layer::new(name.to_string());
        let id = layer.id;
        self.layers.push(layer);
        id
    }

    /// Remove a layer by id. Returns an error if it's the last layer.
    /// Items owned by the removed layer are reassigned to the first remaining layer.
    pub fn remove_layer(&mut self, id: LayerId) -> Result<(), String> {
        if self.layers.len() <= 1 {
            return Err("Cannot remove the last layer".to_string());
        }

        if let Some(index) = self.layers.iter().position(|layer| layer.id == id) {
            self.layers.remove(index);
            
            // Reassign items from the removed layer to the first layer
            let first_layer_id = self.layers[0].id;
            for (_, layer_id) in self.item_layers.iter_mut() {
                if *layer_id == id {
                    *layer_id = first_layer_id;
                }
            }
        }

        Ok(())
    }

    /// Move a layer to a new position in the stack.
    /// Index is clamped to valid range [0, layer_count).
    pub fn move_layer(&mut self, id: LayerId, new_index: usize) {
        if let Some(current_index) = self.layers.iter().position(|layer| layer.id == id) {
            let new_index = new_index.min(self.layers.len().saturating_sub(1));
            let layer = self.layers.remove(current_index);
            self.layers.insert(new_index, layer);
        }
    }

    /// Rename a layer.
    pub fn rename_layer(&mut self, id: LayerId, name: &str) {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == id) {
            layer.name = name.to_string();
        }
    }

    /// Set layer visibility.
    pub fn set_visible(&mut self, id: LayerId, visible: bool) {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == id) {
            layer.visible = visible;
        }
    }

    /// Set layer lock state.
    pub fn set_locked(&mut self, id: LayerId, locked: bool) {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == id) {
            layer.locked = locked;
        }
    }

    /// Check if a layer is visible. Returns true if layer not found (safe default).
    pub fn is_visible(&self, id: LayerId) -> bool {
        self.layers
            .iter()
            .find(|layer| layer.id == id)
            .map(|layer| layer.visible)
            .unwrap_or(true)
    }

    /// Check if a layer is locked. Returns false if layer not found (safe default).
    pub fn is_locked(&self, id: LayerId) -> bool {
        self.layers
            .iter()
            .find(|layer| layer.id == id)
            .map(|layer| layer.locked)
            .unwrap_or(false)
    }

    /// Get a layer by id.
    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|layer| layer.id == id)
    }

    /// Iterate over layers in render order (bottom to top).
    pub fn layers_bottom_to_top(&self) -> impl Iterator<Item = &Layer> {
        self.layers.iter()
    }

    /// Get the number of layers.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Get the z-index (0-based position) of a layer.
    /// Returns None if the layer is not found.
    pub fn z_index(&self, id: LayerId) -> Option<usize> {
        self.layers.iter().position(|layer| layer.id == id)
    }

    /// Get the id of the default (first) layer.
    pub fn default_layer(&self) -> LayerId {
        self.layers[0].id
    }

    /// Assign an item to a layer.
    pub fn set_item_layer(&mut self, item: I, layer: LayerId) {
        self.item_layers.insert(item, layer);
    }

    /// Get the layer that owns an item.
    /// Returns None if the item is not assigned to any layer.
    pub fn get_item_layer(&self, item: &I) -> Option<LayerId> {
        self.item_layers.get(item).copied()
    }

    /// Remove an item from the ownership map.
    pub fn remove_item(&mut self, item: &I) {
        self.item_layers.remove(item);
    }
}

impl<I> Default for LayerStack<I>
where
    I: Eq + std::hash::Hash + Copy + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn layer_id_uniqueness() {
        let id1 = LayerId::new();
        let id2 = LayerId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn layer_id_equality() {
        let uuid = Uuid::new_v4();
        let id1 = LayerId(uuid);
        let id2 = LayerId(uuid);
        assert_eq!(id1, id2);
        
        let mut set = HashSet::new();
        set.insert(id1);
        assert!(set.contains(&id2));
    }

    #[test]
    fn layer_id_display() {
        let id = LayerId::new();
        let display_str = format!("{}", id);
        assert_eq!(display_str.len(), 36); // UUID string length
        
        // Should be able to parse back to UUID
        let parsed = Uuid::parse_str(&display_str);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), id.0);
    }

    #[test]
    fn layer_id_default() {
        let id = LayerId::default();
        // Default should create new UUID
        assert_eq!(id.0.get_version_num(), 4);
    }

    #[test]
    fn layer_new_defaults() {
        let layer = Layer::new("Background");
        assert_eq!(layer.name, "Background");
        assert!(layer.visible);
        assert!(!layer.locked);
        assert_eq!(layer.id.0.get_version_num(), 4);
    }

    #[test]
    fn layer_with_id() {
        let id = LayerId::new();
        let layer = Layer::with_id(id, "Overlay");
        assert_eq!(layer.id, id);
        assert_eq!(layer.name, "Overlay");
        assert!(layer.visible);
        assert!(!layer.locked);
    }

    #[test]
    fn layer_stack_new_has_default_layer() {
        let stack: LayerStack<u32> = LayerStack::new();
        assert_eq!(stack.layer_count(), 1);
        
        let layers: Vec<_> = stack.layers_bottom_to_top().collect();
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].name, "Layer 1");
    }

    #[test]
    fn layer_stack_add_layer() {
        let mut stack: LayerStack<u32> = LayerStack::new();
        let id = stack.add_layer("Layer 2");
        
        assert_eq!(stack.layer_count(), 2);
        
        let layer = stack.get_layer(id);
        assert!(layer.is_some());
        assert_eq!(layer.unwrap().name, "Layer 2");
    }

    #[test]
    fn layer_stack_remove_layer_preserves_minimum() {
        let mut stack: LayerStack<u32> = LayerStack::new();
        let default_id = stack.default_layer();
        
        let result = stack.remove_layer(default_id);
        assert!(result.is_err());
        assert_eq!(stack.layer_count(), 1);
    }

    #[test]
    fn layer_stack_remove_layer_reassigns_items() {
        let mut stack: LayerStack<u32> = LayerStack::new();
        let layer2_id = stack.add_layer("Layer 2");
        let default_id = stack.default_layer();
        
        // Assign item to layer 2
        stack.set_item_layer(42, layer2_id);
        assert_eq!(stack.get_item_layer(&42), Some(layer2_id));
        
        // Remove layer 2
        let result = stack.remove_layer(layer2_id);
        assert!(result.is_ok());
        
        // Item should be reassigned to default layer
        assert_eq!(stack.get_item_layer(&42), Some(default_id));
    }

    #[test]
    fn layer_stack_move_layer() {
        let mut stack: LayerStack<u32> = LayerStack::new();
        let layer2_id = stack.add_layer("Layer 2");
        let layer3_id = stack.add_layer("Layer 3");
        
        // Initially: [Layer 1, Layer 2, Layer 3]
        assert_eq!(stack.z_index(layer2_id), Some(1));
        assert_eq!(stack.z_index(layer3_id), Some(2));
        
        // Move Layer 3 to position 0
        stack.move_layer(layer3_id, 0);
        
        // Now: [Layer 3, Layer 1, Layer 2]
        assert_eq!(stack.z_index(layer3_id), Some(0));
        assert_eq!(stack.z_index(layer2_id), Some(2));
    }

    #[test]
    fn layer_stack_move_layer_clamps_index() {
        let mut stack: LayerStack<u32> = LayerStack::new();
        let layer_id = stack.add_layer("Layer 2");
        
        // Move to index way beyond bounds
        stack.move_layer(layer_id, 100);
        
        // Should be clamped to last position
        assert_eq!(stack.z_index(layer_id), Some(1));
    }

    #[test]
    fn layer_stack_rename_layer() {
        let mut stack: LayerStack<u32> = LayerStack::new();
        let layer_id = stack.add_layer("Original");
        
        stack.rename_layer(layer_id, "Renamed");
        
        let layer = stack.get_layer(layer_id);
        assert!(layer.is_some());
        assert_eq!(layer.unwrap().name, "Renamed");
    }

    #[test]
    fn layer_stack_visibility_toggle() {
        let mut stack: LayerStack<u32> = LayerStack::new();
        let layer_id = stack.add_layer("Test Layer");
        
        // Initially visible
        assert!(stack.is_visible(layer_id));
        
        // Set invisible
        stack.set_visible(layer_id, false);
        assert!(!stack.is_visible(layer_id));
        
        // Set visible again
        stack.set_visible(layer_id, true);
        assert!(stack.is_visible(layer_id));
    }

    #[test]
    fn layer_stack_lock_toggle() {
        let mut stack: LayerStack<u32> = LayerStack::new();
        let layer_id = stack.add_layer("Test Layer");
        
        // Initially unlocked
        assert!(!stack.is_locked(layer_id));
        
        // Set locked
        stack.set_locked(layer_id, true);
        assert!(stack.is_locked(layer_id));
        
        // Set unlocked again
        stack.set_locked(layer_id, false);
        assert!(!stack.is_locked(layer_id));
    }

    #[test]
    fn layer_stack_safe_defaults() {
        let stack: LayerStack<u32> = LayerStack::new();
        let nonexistent_id = LayerId::new();
        
        // Should return safe defaults for nonexistent layers
        assert!(stack.is_visible(nonexistent_id)); // true for safety
        assert!(!stack.is_locked(nonexistent_id)); // false for safety
        assert!(stack.get_layer(nonexistent_id).is_none());
        assert!(stack.z_index(nonexistent_id).is_none());
    }

    #[test]
    fn layer_stack_item_ownership() {
        let mut stack: LayerStack<u32> = LayerStack::new();
        let layer_id = stack.add_layer("Test Layer");
        
        // Initially unassigned
        assert_eq!(stack.get_item_layer(&42), None);
        
        // Assign to layer
        stack.set_item_layer(42, layer_id);
        assert_eq!(stack.get_item_layer(&42), Some(layer_id));
        
        // Remove item
        stack.remove_item(&42);
        assert_eq!(stack.get_item_layer(&42), None);
    }

    #[test]
    fn layer_stack_render_order() {
        let mut stack: LayerStack<u32> = LayerStack::new();
        let layer2_id = stack.add_layer("Layer 2");
        let layer3_id = stack.add_layer("Layer 3");
        
        let layers: Vec<_> = stack.layers_bottom_to_top().collect();
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0].name, "Layer 1");
        assert_eq!(layers[1].name, "Layer 2");
        assert_eq!(layers[2].name, "Layer 3");
        
        // Verify z-index matches iteration order
        assert_eq!(stack.z_index(stack.default_layer()), Some(0));
        assert_eq!(stack.z_index(layer2_id), Some(1));
        assert_eq!(stack.z_index(layer3_id), Some(2));
    }

    #[test]
    fn layer_stack_default_layer() {
        let stack: LayerStack<u32> = LayerStack::new();
        let default_id = stack.default_layer();
        
        let layer = stack.get_layer(default_id);
        assert!(layer.is_some());
        assert_eq!(layer.unwrap().name, "Layer 1");
    }

    #[test]
    fn layer_stack_different_item_types() {
        // Test with string item IDs
        let mut stack: LayerStack<&str> = LayerStack::new();
        let layer_id = stack.add_layer("Test");
        
        stack.set_item_layer("item1", layer_id);
        assert_eq!(stack.get_item_layer(&"item1"), Some(layer_id));
        
        stack.remove_item(&"item1");
        assert_eq!(stack.get_item_layer(&"item1"), None);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn layer_id_serde() {
        let id = LayerId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: LayerId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn layer_serde() {
        let layer = Layer::new("Test Layer");
        let json = serde_json::to_string(&layer).unwrap();
        let deserialized: Layer = serde_json::from_str(&json).unwrap();
        assert_eq!(layer.id, deserialized.id);
        assert_eq!(layer.name, deserialized.name);
        assert_eq!(layer.visible, deserialized.visible);
        assert_eq!(layer.locked, deserialized.locked);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn layer_stack_serde() {
        let mut stack: LayerStack<u32> = LayerStack::new();
        let layer_id = stack.add_layer("Test Layer");
        stack.set_item_layer(42, layer_id);
        
        let json = serde_json::to_string(&stack).unwrap();
        let deserialized: LayerStack<u32> = serde_json::from_str(&json).unwrap();
        
        assert_eq!(stack.layer_count(), deserialized.layer_count());
        assert_eq!(stack.get_item_layer(&42), deserialized.get_item_layer(&42));
    }
}