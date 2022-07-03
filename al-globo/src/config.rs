use std::env::args;
use std::error::Error;
use std::fs::File;
use std::process::exit;

use serde::Deserialize;
use serde_json;

/// Configuración de Entidad Externa a la cual enviar los
#[derive(Deserialize, Debug)]
struct ExternalEntityConfiguration {
    /// Nombre de la entidad externa
    pub(crate) name: String,
    /// Dirección IP
    pub(crate) ip: String,
    /// Puerto
    pub(crate) port: String,
}

/// Configuración del Procesador de Pagos
#[derive(Deserialize, Debug)]
pub(crate) struct Configuration {
    /// Configuración de las Entidades Externas a
    /// las cuales se conecta el Procesador de pagos
    pub(crate) external_entities: Vec<ExternalEntityConfiguration>,
    /// Ruta del archivo de *logs*
    pub(crate) log_file: String,
    /// Ruta del archivo de entrada de los pagos a procesar
    pub(crate) payment_transactions: String,
    /// Ruta del archivo de salida de los pagos fallidos
    pub(crate) failed_transactions: String,
}

impl Configuration {
    /// Obtener configuración del servidor a partir de un archivo `json`
    /// el cual debe cumplir con el formato:
    /// ```json
    /// {
    ///   "external_entities": [
    ///     {
    ///       "name": "string",
    ///       "ip": "string",
    ///       "port": "string"
    ///     }
    ///   ],
    ///   "log_file": "string",
    ///   "payment_transactions": "string",
    ///   "failed_transactions": "string"
    /// }
    /// ```
    pub(crate) fn new(path: &str) -> Result<Configuration, Box<dyn Error>> {
        let f = File::open(path)?;
        let config = serde_json::from_reader(f)?;

        Ok(config)
    }
}
