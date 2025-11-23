Here is the formal documentation for your Proof of Concept. It is designed to be shared with your team to explain the architectural decisions, the hybrid data strategy, and the setup instructions.

---

# **Logistics Analytics Dashboard: Proof of Concept (PoC)**

## **1\. Executive Summary**

This Proof of Concept demonstrates a high-performance, hybrid architecture for a logistics data analytics dashboard. It addresses the requirement to handle massive datasets without overwhelming the browser, while maintaining instant interactivity for smaller, aggregated datasets.

**Key Differentiator:** The application implements a "Hybrid Data Strategy" that seamlessly switches between **Client-Side Processing** (for summary data) and **Server-Side Processing** (for massive granular data) within the same user interface.

---

## **2\. Architecture Overview**

### **The Tech Stack**

* **Backend:** **Rust** (High-performance systems language)  
  * *Framework:* Tonic (gRPC implementation) \+ Tonic-Web (Browser compatibility).  
  * *Role:* Data generation, heavy sorting, filtering, and aggregation.  
* **Protocol:** **gRPC-Web** (Binary over HTTP)  
  * *Role:* Strongly typed, bandwidth-efficient communication between browser and server.  
  * *Schema:* Defined via Protocol Buffers (.proto).  
* **Frontend:** **TypeScript** \+ **Next.js** (React)  
  * *Client:* ConnectRPC (Modern gRPC client for web).  
  * *State/UI:* TanStack Table (Headless UI library for data grids).

### **Architecture Diagram**

Code snippet

graph LR  
    subgraph Browser \[Frontend: Next.js\]  
        UI\[User Interface\]  
        RegionTable\[Region Table \<br/\> (Client-Side Sort)\]  
        ShipmentTable\[Shipment Table \<br/\> (Server-Side Sort)\]  
        Connect\[ConnectRPC Client\]  
    end

    subgraph Network \[gRPC-Web\]  
        Proto\[Binary Stream\]  
    end

    subgraph Server \[Backend: Rust\]  
        Tonic\[Tonic gRPC Server\]  
        MockDB\[Mock Database \<br/\> (10k Rows RAM)\]  
    end

    UI \--\> RegionTable  
    UI \--\> ShipmentTable  
    RegionTable \-- "Fetch Once (All Data)" \--\> Connect  
    ShipmentTable \-- "Fetch Page/Sort" \--\> Connect  
    Connect \<--\> Proto \<--\> Tonic  
    Tonic \<--\> MockDB

---

## **3\. The Hybrid Data Strategy**

This PoC specifically demonstrates two distinct patterns for handling data, proving that we are not locked into a single approach.

### **Pattern A: "Small Data" (Region Stats)**

* **Use Case:** High-level aggregations, summaries, or datasets \< 5,000 rows.  
* **Mechanism:** The server calculates the aggregation *once* and sends the complete list to the browser.  
* **User Experience:** Sorting and filtering are **instant** (0ms latency) because they occur in the browser's CPU. No network requests are made after the initial load.

### **Pattern B: "Big Data" (Shipment Manifest)**

* **Use Case:** Granular records, logs, or datasets \> 10,000 rows (potentially millions).  
* **Mechanism:** The browser acts as a "remote control." It holds only the data currently visible (e.g., 50 rows).  
* **User Experience:** When the user sorts by "Weight," the browser sends a request to the server. The server sorts the full dataset and returns only the top 50 results. This ensures the browser never crashes from memory overload.

---

## **4\. Project Structure**

Plaintext

/logistics-dashboard  
├── /proto                   \# SHARED CONTRACT  
│   └── dashboard.proto      \# Defines API methods and Data Types  
├── /backend                 \# RUST SERVER  
│   ├── Cargo.toml           \# Dependencies (Tonic, Tokio, Tower)  
│   ├── build.rs             \# Auto-generates Rust code from .proto  
│   └── src/main.rs          \# Implementation of Sorting/Filtering logic  
└── /frontend                \# NEXT.JS CLIENT  
    ├── buf.gen.yaml         \# Config for TypeScript generation  
    ├── package.json         \# Dependencies (ConnectRPC, TanStack Table)  
    ├── app/page.tsx         \# Main UI combining both tables  
    └── components/          \# Smart Table Components

---

## **5\. Implementation Details**

### **The Schema (dashboard.proto)**

This file is the single source of truth. If this file changes, both the Backend and Frontend builds will fail, ensuring type safety.

Protocol Buffers

service AnalyticsService {  
  // Pattern B: Big Data (Server does the work)  
  rpc GetShipments (ViewRequest) returns (ViewResponse);  
    
  // Pattern A: Small Data (Client does the work)  
  rpc GetRegionStats (Empty) returns (RegionList);  
}

### **The Rust Backend (backend/src/main.rs)**

* **Mock Data:** Generates 10,000 random shipments on startup.  
* **CORS:** Configured using tower-http to allow the Next.js frontend to read gRPC headers (grpc-status).  
* **Logic:** Implements a custom sort comparator to handle dynamic column sorting (e.g., sorting by Timestamp vs. Weight) on the server.

### **The Frontend Client (frontend/lib/client.ts)**

Uses **ConnectRPC** to communicate. This abstracts away the binary parsing.

TypeScript

// Example usage  
const response \= await client.getShipments({   
  pageNumber: 1,   
  sort: { columnId: "cargoWeightKg", isAscending: false }   
});

---

## **6\. Setup & Run Guide**

### **Prerequisites**

1. **Rust:** Install via rustup.  
2. **Node.js:** Install Node v18+.  
3. **Buf:** (Optional but recommended) for efficient code generation, though npm scripts handle it in this PoC.

### **Step 1: Start the Backend**

The backend must be running to serve gRPC requests.

Bash

cd backend  
cargo run  
\# Expected Output: Server listening on 0.0.0.0:50051

### **Step 2: Start the Frontend**

Open a new terminal window.

Bash

cd frontend  
npm install  
npm run dev  
\# Expected Output: Ready in 2.3s. Listening on http://localhost:3000

### **Step 3: Verify**

Navigate to http://localhost:3000.

1. **Test Region Table:** Click the "Revenue" header. It should sort instantly. Check the Backend terminal—you should see **no** new logs.  
2. **Test Shipment Table:** Click the "Weight" header. It should take a brief moment. Check the Backend terminal—you should see a log entry: Request received for page: 1\.

---

## **7\. Future Considerations (Post-PoC)**

* **Database:** Replace the in-memory Vec\<MockShipment\> with a connection pool to PostgreSQL (using sqlx) or ClickHouse for analytics.  
* **Filtering:** The .proto file already supports FilterCriteria. We can implement SQL WHERE clause generation in Rust to support complex filtering.  
* **Streaming:** For data exports (e.g., "Download CSV"), we can utilize gRPC Server Streaming to send data in chunks without buffering.