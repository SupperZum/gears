pub mod messages;
pub mod primitives;
pub mod tx;
pub mod value_renderer;

#[cfg(test)]
pub(super) mod test_functions {
    use crate::types::{
        denom::Denom,
        tx::metadata::{DenomUnit, Metadata},
    };

    pub fn get_metadata(denom: &Denom) -> Option<Metadata> {
        match denom.to_string().as_str() {
            "uatom" => Some(Metadata {
                description: String::new(),
                denom_units: vec![
                    DenomUnit {
                        denom: "ATOM".parse().expect("this is a valid denom"),
                        exponent: 6,
                        aliases: Vec::new(),
                    },
                    DenomUnit {
                        denom: "uatom".parse().expect("this is a valid denom"),
                        exponent: 0,
                        aliases: Vec::new(),
                    },
                ],
                base: "uatom".into(),
                display: "ATOM".into(),
                name: String::new(),
                symbol: String::new(),
            }),
            "uon" => Some(Metadata {
                description: String::new(),
                denom_units: vec![
                    DenomUnit {
                        denom: "AAUON".parse().expect("this is a valid denom"),
                        exponent: 6,
                        aliases: Vec::new(),
                    },
                    DenomUnit {
                        denom: "uon".parse().expect("this is a valid denom"),
                        exponent: 0,
                        aliases: Vec::new(),
                    },
                ],
                base: "uon".into(),
                display: "AAUON".into(),
                name: String::new(),
                symbol: String::new(),
            }),
            _ => None,
        }
    }
}
