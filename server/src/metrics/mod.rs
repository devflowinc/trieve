use prometheus::{Encoder, Error, Gauge, Registry};

#[derive(Clone)]
pub struct Metrics {
    registry: Registry,
    pub ingest_queue_gauge: Gauge,
}

// Create a metrics system
impl Metrics {
    pub fn new() -> Result<Self, Error> {
        let registry = Registry::new();

        let ingest_queue_gauge = Gauge::new("ingest_queue", "number of items in the ingest queue")?;
        registry.register(Box::new(ingest_queue_gauge.clone()))?;
        Ok(Metrics {
            registry,
            ingest_queue_gauge,
        })
    }

    pub fn get_response(&self) -> String {
        let mut buffer = vec![];
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}
