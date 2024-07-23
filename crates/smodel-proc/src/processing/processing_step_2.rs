use crate::*;

pub struct ProcessingStep2();

impl ProcessingStep2 {
    pub fn exec(&self, host: &mut SModelHost, m: &Rc<SmType>) -> bool {
        // 1. Create a SmTypeSlot.
        let slot = host.factory.create_smtype_slot(m.name.to_string());

        // 1.2. Resolve the inherited base.
        // 1.3. If the inherited base failed to resolve, ignore that type
        // (assuming the error was reported); otherwise
        // 1.3.1. Contribute the type to the inherited base's list of subtypes.
        if let Some(inherits) = &m.inherits {
            if let Some(inherited_smtype) = host.smtype_slots.get(&inherits.to_string()) {
                slot.set_inherits(Some(inherited_smtype));
                inherited_smtype.subtypes().push(slot.clone());
            } else {
                inherits.span().unwrap().error(format!("Data type '{}' not found.", inherits.to_string())).emit();
                return false;
            }
        }

        // 1.4. Contribute type slot to the set of known type slots.
        if host.smtype_slots.contains_key(&slot.name()) {
            m.name.span().unwrap().error(format!("Redefining '{}'", slot.name())).emit();
            return false;
        } else {
            host.smtype_slots.insert(slot.name(), slot.clone());
        }

        // 1.5. Map the data type node to the data type slot.
        host.semantics.set(m, Some(slot));
        true
    }
}