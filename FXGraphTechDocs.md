

# **Technical Specification: Logistics Analytics Dashboard (PoC)**

Version: 1.0 (PoC)  
Date: November 23, 2025  
Architecture: Hybrid Client/Server Data Processing via gRPC-Web.

---

## **1\. Technology Stack**

### **Backend (Service Layer)**

* **Runtime:** Rust (Edition 2021\)  
* **Framework:** tonic (v0.12) \- Native gRPC implementation over HTTP/2.  
* **Web Interop:** tonic-web (v0.12) \- gRPC-Web translation layer for browser compatibility.  
* **Async Runtime:** tokio (v1.0+) \- Multi-threaded asynchronous execution.  
* **Middleware:** tower-http (v0.5) \- Handles CORS and HTTP layer instrumentation.  
* **Serialization:** prost (v0.13) \- Protocol Buffers implementation for Rust.

### **Frontend (Presentation Layer)**

* **Framework:** Next.js 14+ (React 18\)  
* **Language:** TypeScript 5+  
* **Network Client:** ConnectRPC (@connectrpc/connect) \- Type-safe gRPC client.  
* **Transport:** @connectrpc/connect-web \- Implements the gRPC-Web protocol over HTTP/1.1.  
* **State/UI Logic:** TanStack Table v8 (@tanstack/react-table) \- Headless data grid.

### **Interface Definition**

* **IDL:** Protocol Buffers v3 (proto3)  
* **Transport Protocol:** gRPC-Web (application/grpc-web+proto)

---

## **2\. Directory Structure & Key Files**

Plaintext

/root  
├── /proto  
│   └── dashboard.proto       \# Single Source of Truth (Schema)  
├── /backend  
│   ├── build.rs              \# Pre-build script: Compiles .proto to Rust  
│   ├── Cargo.toml            \# Rust dependencies  
│   └── src/main.rs           \# Server entry point, Mock DB, Business Logic  
└── /frontend  
    ├── buf.gen.yaml          \# Buf configuration for TS generation  
    ├── lib/client.ts         \# ConnectRPC Transport singleton  
    ├── components/  
    │   ├── DashboardTable.tsx \# Server-Side Sort Implementation  
    │   └── RegionTable.tsx    \# Client-Side Sort Implementation  
    └── gen/                  \# Auto-generated TS types (DO NOT EDIT)

---

## **3\. Interface Specification (dashboard.proto)**

The API defines two distinct data access patterns.

Protocol Buffers

syntax \= "proto3";  
package dashboard;

service AnalyticsService {  
  // Pattern A: Server-Side Processing (Pagination/Sorting)  
  rpc GetShipments (ViewRequest) returns (ViewResponse);

  // Pattern B: Client-Side Processing (Snapshot)  
  rpc GetRegionStats (Empty) returns (RegionList);  
}

message ViewRequest {  
  int32 page\_number \= 1;  
  int32 items\_per\_page \= 2;  
  SortOption sort \= 3;        // Optional: if null, default sort applies  
  repeated FilterCriteria filters \= 4;  
}

message SortOption {  
  string column\_id \= 1;  
  bool is\_ascending \= 2;  
}

// ... (See source for full message definitions)

---

## **4\. Backend Implementation Details**

### **4.1 Server Configuration (main.rs)**

The server enables the GrpcWebLayer to translate browser HTTP/1.1 requests into HTTP/2 gRPC calls.

Crucial Layer Ordering:  
Middleware layers are wrapped outside-in. CORS must be the outermost layer to handle preflight OPTIONS requests before they reach the gRPC logic.

Rust

Server::builder()  
    .accept\_http1(true)                   // Required for gRPC-Web  
    .layer(cors\_layer)                    // 1\. Handle CORS  
    .layer(GrpcWebLayer::new())           // 2\. Translate gRPC-Web \-\> gRPC  
    .add\_service(AnalyticsServiceServer::new(service))  
    .serve(addr)  
    .await?;

### **4.2 CORS Configuration**

Browsers enforce strict CORS for gRPC-Web. The tower-http configuration must expose specific gRPC headers, otherwise the client will throw generic protocol errors even on successful requests.

* **Allowed Origin:** MirrorRequest (or explicit localhost:3000)  
* **Exposed Headers:** grpc-status, grpc-message, grpc-status-details-bin

### **4.3 Data Logic**

* **Mock Data:** Vec\<MockShipment\> generated via rand at startup (10k rows).  
* **get\_shipments (Server Sort):**  
  1. Clones the in-memory vector.  
  2. Performs sort\_by based on request.sort.column\_id.  
  3. Applies skip((page-1) \* limit).take(limit).  
  4. Returns ViewResponse.  
* **get\_region\_stats (Client Sort):**  
  1. Returns a static Vec\<RegionStat\>.  
  2. No sorting/paging logic applied serverside.

---

## **5\. Frontend Implementation Details**

### **5.1 Code Generation**

TypeScript types are generated using buf (or protoc-gen-es).

* **Command:** npx buf generate ../proto  
* **Output:**  
  * \*\_pb.ts: TypeScript interfaces for messages.  
  * \*\_connect.ts: Service definition and method signatures.

### **5.2 ConnectRPC Client (lib/client.ts)**

Instantiates a singleton client using createGrpcWebTransport.

* **BaseUrl:** http://localhost:50051  
* **Transport:** Binary format (default).

### **5.3 TanStack Table Configuration**

Scenario A: Server-Side (Big Data)  
File: DashboardTable.tsx

* **Mode:** Manual.  
* **Configuration:**  
  TypeScript  
  manualPagination: true,  
  manualSorting: true,

* **Effect:** Sort/Page state changes trigger useEffect \-\> analyticsClient.getShipments. Table does *not* sort the data array locally.

Scenario B: Client-Side (Small Data)  
File: RegionTable.tsx

* **Mode:** Automatic.  
* **Configuration:**  
  TypeScript  
  getCoreRowModel: getCoreRowModel(),  
  getSortedRowModel: getSortedRowModel(), // Enables local sorting  
  // manualSorting is OMITTED/FALSE

* **Effect:** useEffect runs once on mount. Sort state changes are handled entirely by the JS engine via getSortedRowModel.

---

## **6\. Build & Run Instructions**

### **6.1 Backend**

**Build Requirement:** protoc (Protocol Buffers Compiler) must be in $PATH, or available via tonic-build.

Bash

cd backend  
\# Compiles .proto via build.rs and starts server  
cargo run

### **6.2 Frontend**

**Build Requirement:** Node.js v18+.

Bash

cd frontend  
\# 1\. Install Deps  
npm install  
\# 2\. Generate TS Client (if proto changed)  
npx buf generate ../proto  
\# 3\. Start Dev Server  
npm run dev

## **7\. Known Limitations (PoC)**

1. **Data Persistence:** Backend uses ephemeral in-memory Vec. Restarting the server resets data.  
2. **Error Handling:** Basic Result types. Production should map Results to specific gRPC Status Codes (e.g., Status::invalid\_argument).  
3. **Security:** No TLS/SSL implemented. Traffic is plaintext HTTP. Production requires a reverse proxy (Nginx/Envoy) or Rust TLS configuration.