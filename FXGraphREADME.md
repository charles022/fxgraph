# **Logistics Analytics Dashboard (PoC)**

**Version:** 1.0

**Architecture:** Hybrid Client/Server Processing via gRPC-Web

## **📖 Overview**

This Proof of Concept (PoC) demonstrates a high-performance, type-safe architecture for logistics analytics. It specifically validates a **Hybrid Data Strategy**, proving that a single dashboard can seamlessly handle two distinct data patterns side-by-side:

1. **Pattern A (Big Data / Server-Side):** Handling massive datasets (10k+ rows) where sorting, filtering, and pagination must happen on the server to save bandwidth and browser memory.  
2. **Pattern B (Small Data / Client-Side):** Handling summary datasets where the server sends a snapshot once, and the browser handles instant, zero-latency sorting.

## **🏗 Tech Stack**

| Layer | Technology | Role |
| :---- | :---- | :---- |
| **Backend** | **Rust** \+ tonic | High-performance gRPC server & data processing. |
| **Middleware** | tonic-web | Translates gRPC to gRPC-Web for browser compatibility. |
| **Protocol** | **Protocol Buffers** | Schema-driven, strictly typed binary communication. |
| **Frontend** | **Next.js** \+ TypeScript | Modern React framework. |
| **Network** | ConnectRPC | Type-safe gRPC client for the browser. |
| **UI State** | TanStack Table | Headless data grid handling both manual (server) and automatic (client) state. |

## **🚀 Getting Started**

### **Prerequisites**

* **Rust:** [Install Rust via rustup](https://rustup.rs/) (cargo 1.70+).  
* **Node.js:** [Install Node.js](https://nodejs.org/) (v18+ recommended).  
* **Protoc:** The Protocol Buffers compiler is required for the Rust build script.  
  * *MacOS:* brew install protobuf  
  * *Linux:* apt install protobuf-compiler  
  * *Windows:* [Install protoc](https://grpc.io/docs/protoc-installation/) and add to PATH.

### **1\. The Schema (Source of Truth)**

The API contract is defined in proto/dashboard.proto. Both the backend and frontend code are auto-generated from this file.

### **2\. Run the Backend**

The backend generates 10,000 rows of mock data in memory on startup.

cd backend  
cargo run

*Expected Output:*

Initialized Mock Database with 10000 rows.  
gRPC Server listening on 0.0.0.0:50051

### **3\. Run the Frontend**

Open a new terminal window.

cd frontend

\# 1\. Install dependencies  
npm install

\# 2\. Generate TypeScript client code from the .proto file  
npm run generate  
\# (Or: npx buf generate ../proto)

\# 3\. Start the Next.js dev server  
npm run dev

*Expected Output:*

Ready in 2.3s  
Listening on http://localhost:3000

## **🧪 Testing the PoC**

Open your browser to http://localhost:3000. You will see two tables.

### **Test 1: The "Small Data" Table (Top Table)**

* **Context:** Represents a regional summary (aggregation).  
* **Action:** Click the **Revenue** header.  
* **Observation:** The table sorts **instantly**.  
* **Backend Check:** Look at your Rust terminal. **No new logs appear**.  
* **Why:** The data was fetched once on load. The browser's JavaScript engine handles the sorting locally.

### **Test 2: The "Big Data" Table (Bottom Table)**

* **Context:** Represents a live manifest of 10,000 shipments.  
* **Action:** Click the **Weight (kg)** header.  
* **Observation:** The table updates after a brief network delay (simulated).  
* **Backend Check:** Look at your Rust terminal. You will see: Processing Server-Side Request: Page 1\.  
* **Why:** The browser sent a gRPC request saying "Give me Page 1, Sorted by Weight." The Rust server sorted the 10k rows in memory and returned the top 10\.

## **📂 Project Structure**

/  
├── proto/                  \# SHARED CONTRACT  
│   └── dashboard.proto     \# The single source of truth  
│  
├── backend/                \# RUST SERVER  
│   ├── src/main.rs         \# Logic for Server-Side Sort & Mock DB  
│   └── build.rs            \# Compiles .proto to Rust code  
│  
└── frontend/               \# NEXT.JS CLIENT  
    ├── components/  
    │   ├── DashboardTable  \# Implements Server-Side Pattern  
    │   └── RegionTable     \# Implements Client-Side Pattern  
    ├── lib/client.ts       \# ConnectRPC configuration  
    └── buf.gen.yaml        \# TypeScript generation config

## **🔧 Troubleshooting**

* **CORS Errors:** Ensure the backend is running. The Rust server is explicitly configured to allow requests from localhost and expose grpc-status headers.  
* **Type Errors:** If you modify dashboard.proto, you **must** restart the backend (to recompile Rust types) and run npm run generate in the frontend (to recompile TypeScript types).