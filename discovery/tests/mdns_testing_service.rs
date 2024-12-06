use discovery::MDNSService;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdns_service_creation() {
        let service = MDNSService::new(
            "ExampleService",
            "_example._tcp",
            "example.local",
            8080,
            Some(b"version=1.0".to_vec()),
        );
        assert_eq!(service.service_name.to_string(), "ExampleService");
        assert_eq!(service.service_type.to_string(), "_example._tcp");
        assert_eq!(service.hostname.to_string(), "example.local");
    }
}
