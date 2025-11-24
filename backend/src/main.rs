use tonic::{transport::Server, Request, Response, Status};
use tonic_web::GrpcWebLayer;
use tower_http::cors::{CorsLayer, AllowOrigin, Any};
use http::Method;
use rand::Rng;

// Import generated code from build.rs
pub mod dashboard {
    tonic::include_proto!("dashboard");
}
use dashboard::analytics_service_server::{AnalyticsService, AnalyticsServiceServer};
use dashboard::{ViewRequest, ViewResponse, RegionList, RegionStat, ShipmentRow, Empty};
use dashboard::{Location, LocationList, FacilityRequest, FacilityStats, WeeklyVolume};

#[derive(Debug, Clone)]
struct MockShipment {
    id: String,
    container_number: String,
    status: String,
    cargo_weight_kg: f64,
    arrival_timestamp: i64,
}

// The "Database" struct
pub struct MyAnalytics {
    db_data: Vec<MockShipment>,
}

impl MyAnalytics {
    // Initialize with 10,000 rows of random data
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut data = Vec::new();
        let statuses = vec!["On Time", "Delayed", "Customs Hold", "Arrived"];

        for i in 0..10000 {
            data.push(MockShipment {
                id: format!("uuid-{}", i),
                container_number: format!("CONT-{:05}", i),
                status: statuses[rng.gen_range(0..4)].to_string(),
                cargo_weight_kg: rng.gen_range(1000.0..50000.0),
                arrival_timestamp: 1700000000 + rng.gen_range(0..10000000),
            });
        }
        println!("Initialized Mock Database with {} rows.", data.len());
        Self { db_data: data }
    }
}

#[tonic::async_trait]
impl AnalyticsService for MyAnalytics {
    
    // --- PATTERN B: Client-Side Processing ---
    async fn get_region_stats(&self, _req: Request<Empty>) -> Result<Response<RegionList>, Status> {
        // Static aggregations (mocking a GROUP BY query)
        let regions = vec![
            RegionStat { region_name: "North America".into(), active_shipments: 450, total_revenue: 125000.0 },
            RegionStat { region_name: "Europe".into(), active_shipments: 320, total_revenue: 98000.0 },
            RegionStat { region_name: "Asia Pacific".into(), active_shipments: 890, total_revenue: 450000.0 },
            RegionStat { region_name: "Latin America".into(), active_shipments: 150, total_revenue: 42000.0 },
        ];

        Ok(Response::new(RegionList { regions }))
    }

    async fn get_locations(&self, _req: Request<Empty>) -> Result<Response<LocationList>, Status> {
        let locations = vec![
            Location { id: "loc-1".into(), name: "New York".into(), latitude: 40.7128, longitude: -74.0060 },
            Location { id: "loc-2".into(), name: "Los Angeles".into(), latitude: 34.0522, longitude: -118.2437 },
            Location { id: "loc-3".into(), name: "Chicago".into(), latitude: 41.8781, longitude: -87.6298 },
            Location { id: "loc-4".into(), name: "Houston".into(), latitude: 29.7604, longitude: -95.3698 },
            Location { id: "loc-5".into(), name: "Denver".into(), latitude: 39.7392, longitude: -104.9903 },
        ];
        Ok(Response::new(LocationList { locations }))
    }

    async fn get_facility_stats(&self, request: Request<FacilityRequest>) -> Result<Response<FacilityStats>, Status> {
        let req = request.into_inner();
        let facility_id = req.facility_id;
        let mut rng = rand::thread_rng();
        
        let mut weeks = Vec::new();
        for w in 1..=4 {
            let mut daily_volumes = Vec::new();
            for _ in 0..7 {
                daily_volumes.push(rng.gen_range(100..1000));
            }
            weeks.push(WeeklyVolume { week_number: w, daily_volumes });
        }

        Ok(Response::new(FacilityStats {
            facility_id,
            weeks,
        }))
    }

    // --- PATTERN A: Server-Side Processing ---
    async fn get_shipments(&self, request: Request<ViewRequest>) -> Result<Response<ViewResponse>, Status> {
        let req = request.into_inner();
        println!("Processing Server-Side Request: Page {}", req.page_number);

        let mut filtered_data = self.db_data.clone();

        // 1. Server-Side Sort
        if let Some(sort_option) = req.sort {
            filtered_data.sort_by(|a, b| {
                let order = match sort_option.column_id.as_str() {
                    "cargoWeightKg" => a.cargo_weight_kg.partial_cmp(&b.cargo_weight_kg).unwrap(),
                    "arrivalTimestamp" => a.arrival_timestamp.cmp(&b.arrival_timestamp),
                    "status" => a.status.cmp(&b.status),
                    _ => a.container_number.cmp(&b.container_number),
                };
                if sort_option.is_ascending { order } else { order.reverse() }
            });
        }

        // 2. Server-Side Pagination
        let page = req.page_number.max(1) as usize;
        let limit = req.items_per_page.max(1) as usize;
        let start = (page - 1) * limit;
        
        let page_data = filtered_data.into_iter().skip(start).take(limit).map(|row| {
            ShipmentRow {
                id: row.id,
                container_number: row.container_number,
                status: row.status,
                cargo_weight_kg: row.cargo_weight_kg,
                arrival_timestamp: row.arrival_timestamp,
                origin_port: "Shanghai".into(),
                destination_port: "Los Angeles".into(),
            }
        }).collect();

        Ok(Response::new(ViewResponse {
            rows: page_data,
            total_row_count: 10000,
            total_pages: (10000 / limit) as i32,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let service = MyAnalytics::new();

    println!("gRPC Server listening on {}", addr);

    // Strict CORS Configuration
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request()) 
        .allow_methods([Method::POST])
        .allow_headers(Any)
        // Expose gRPC specific headers so the browser client doesn't fail
        .expose_headers([
            "grpc-status".parse().unwrap(), 
            "grpc-message".parse().unwrap(), 
            "grpc-status-details-bin".parse().unwrap()
        ]);

    Server::builder()
        .accept_http1(true)         // Enable HTTP/1.1 support
        .layer(cors)                // Handle CORS first
        .layer(GrpcWebLayer::new()) // Translate gRPC-Web second
        .add_service(AnalyticsServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
