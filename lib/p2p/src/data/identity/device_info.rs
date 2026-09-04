use serde::{Deserialize, Serialize};

use crate::infra::error::DeviceInfoError;

/// Informações de identificação do dispositivo (sistema operacional, nome e versão do app).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Nome do sistema operacional (ex: "windows", "linux", "android").
    pub os: String,
    /// Nome amigável ou hostname do dispositivo.
    pub name: String,
    /// Versão do aplicativo executado no dispositivo.
    pub version: String,
}

/// Contrato para provedores de informações do dispositivo local.
pub trait DeviceInfoProvider {
    /// Retorna as informações do dispositivo atual ou um erro caso não seja possível obtê-las.
    fn provide(&self) -> Result<DeviceInfo, DeviceInfoError>;
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn device_info_serialization_roundtrip_proptest(
            operating_system in ".*",
            device_name in ".*",
            app_version in ".*"
        ) {
            let original_info = DeviceInfo {
                os: operating_system,
                name: device_name,
                version: app_version,
            };

            let json_representation =
                serde_json::to_string(&original_info).expect("Failed to serialize DeviceInfo");
            let deserialized_info: DeviceInfo =
                serde_json::from_str(&json_representation).expect("Failed to deserialize DeviceInfo");

            prop_assert_eq!(original_info, deserialized_info);
        }
    }

    #[test]
    fn serialization_and_deserialization_roundtrip() {
        // Verifica se a serialização JSON e a desserialização recompõem o objeto exatamente igual
        let original_info = DeviceInfo {
            os: "linux".to_string(),
            name: "workstation-01".to_string(),
            version: "1.2.3".to_string(),
        };

        let json_representation =
            serde_json::to_string(&original_info).expect("Failed to serialize DeviceInfo");
        let deserialized_info: DeviceInfo =
            serde_json::from_str(&json_representation).expect("Failed to deserialize DeviceInfo");

        assert_eq!(original_info, deserialized_info);
    }

    #[test]
    fn empty_fields_serialize_correctly() {
        // Garante que campos de texto vazios são serializados corretamente sem panic ou perda de chaves
        let empty_info =
            DeviceInfo { os: String::new(), name: String::new(), version: String::new() };

        let json_representation =
            serde_json::to_string(&empty_info).expect("Failed to serialize empty DeviceInfo");
        assert!(json_representation.contains("\"os\":\"\""));
        assert!(json_representation.contains("\"name\":\"\""));
        assert!(json_representation.contains("\"version\":\"\""));

        let deserialized_info: DeviceInfo = serde_json::from_str(&json_representation)
            .expect("Failed to deserialize empty DeviceInfo");
        assert_eq!(empty_info, deserialized_info);
    }
}
