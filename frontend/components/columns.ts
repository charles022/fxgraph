import { createColumnHelper } from "@tanstack/react-table";
import { ShipmentRow } from "@/gen/dashboard_pb"; 

const columnHelper = createColumnHelper<ShipmentRow>();

export const shipmentColumns = [
  columnHelper.accessor("containerNumber", {
    header: "Container ID",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("status", {
    header: "Status",
    cell: (info) => {
        const val = info.getValue();
        const color = val === "Delayed" ? "text-red-600" : "text-green-600";
        return <span className={`font-medium ${color}`}>{val}</span>;
    },
  }),
  columnHelper.accessor("cargoWeightKg", {
    header: "Weight (kg)",
    cell: (info) => info.getValue().toLocaleString(),
  }),
  columnHelper.accessor("arrivalTimestamp", {
    header: "Arrival Date",
    // Convert Unix timestamp (BigInt) to Number for Date constructor
    cell: (info) => new Date(Number(info.getValue()) * 1000).toLocaleDateString(),
  }),
];
