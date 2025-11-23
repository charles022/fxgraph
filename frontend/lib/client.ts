import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { createClient } from "@connectrpc/connect";
import { AnalyticsService } from "../gen/dashboard_connect"; 

// This transport speaks "application/grpc-web" over HTTP/1.1
const transport = createGrpcWebTransport({
  baseUrl: "http://localhost:50051",
});

export const analyticsClient = createClient(AnalyticsService, transport);
