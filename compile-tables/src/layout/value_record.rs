//! The ValueRecord type used in the GPOS table

use font_tables::compile::{FontWrite, TableWriter};

use super::gpos::ValueFormat;

#[derive(Clone, Default, PartialEq)]
pub struct ValueRecord {
    pub x_placement: Option<i16>,
    pub y_placement: Option<i16>,
    pub x_advance: Option<i16>,
    pub y_advance: Option<i16>,
    pub x_placement_device: Option<i16>,
    pub y_placement_device: Option<i16>,
    pub x_advance_device: Option<i16>,
    pub y_advance_device: Option<i16>,
}

impl ValueRecord {
    /// The [ValueFormat] of this record.
    pub fn format(&self) -> ValueFormat {
        macro_rules! flag_if_true {
            ($field:expr, $flag:expr) => {
                $field
                    .is_some()
                    .then(|| $flag)
                    .unwrap_or(ValueFormat::empty())
            };
        }

        flag_if_true!(self.x_placement, ValueFormat::X_PLACEMENT)
            | flag_if_true!(self.y_placement, ValueFormat::Y_PLACEMENT)
            | flag_if_true!(self.x_advance, ValueFormat::X_ADVANCE)
            | flag_if_true!(self.y_advance, ValueFormat::Y_ADVANCE)
            | flag_if_true!(self.x_placement_device, ValueFormat::X_PLACEMENT_DEVICE)
            | flag_if_true!(self.y_placement_device, ValueFormat::Y_PLACEMENT_DEVICE)
            | flag_if_true!(self.x_advance_device, ValueFormat::X_ADVANCE_DEVICE)
            | flag_if_true!(self.y_advance_device, ValueFormat::Y_ADVANCE_DEVICE)
    }
}

impl FontWrite for ValueRecord {
    fn write_into(&self, writer: &mut TableWriter) {
        self.x_placement.map(|v| v.write_into(writer));
        self.y_placement.map(|v| v.write_into(writer));
        self.x_advance.map(|v| v.write_into(writer));
        self.y_advance.map(|v| v.write_into(writer));
        self.x_placement_device.map(|v| v.write_into(writer));
        self.y_placement_device.map(|v| v.write_into(writer));
        self.x_advance_device.map(|v| v.write_into(writer));
        self.y_advance_device.map(|v| v.write_into(writer));
    }
}

impl std::fmt::Debug for ValueRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut f = f.debug_struct("ValueRecord");
        self.x_placement.map(|x| f.field("x_placement", &x));
        self.y_placement.map(|y| f.field("y_placement", &y));
        self.x_advance.map(|x| f.field("x_advance", &x));
        self.y_advance.map(|y| f.field("y_advance", &y));
        self.x_placement_device
            .map(|x| f.field("x_placement_device", &x));
        self.y_placement_device
            .map(|y| f.field("y_placement_device", &y));
        self.x_advance_device
            .map(|x| f.field("x_advance_device", &x));
        self.y_advance_device
            .map(|y| f.field("y_advance_device", &y));
        f.finish()
    }
}