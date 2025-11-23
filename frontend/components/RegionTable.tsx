"use client";
import { useState, useEffect } from "react";
import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel, // Client-side logic
  flexRender,
  SortingState,
  createColumnHelper,
} from "@tanstack/react-table";
import { analyticsClient } from "@/lib/client";
import { RegionStat } from "@/gen/dashboard_pb";
import { ArrowUpDown } from "lucide-react";

const columnHelper = createColumnHelper<RegionStat>();
const columns = [
  columnHelper.accessor("regionName", { header: "Region" }),
  columnHelper.accessor("activeShipments", { header: "Active Shipments" }),
  columnHelper.accessor("totalRevenue", { 
    header: "Revenue",
    cell: (info) => `$${info.getValue().toLocaleString()}`
  }),
];

export default function RegionTable() {
  const [data, setData] = useState<RegionStat[]>([]);
  const [sorting, setSorting] = useState<SortingState>([]);

  useEffect(() => {
    async function load() {
      const res = await analyticsClient.getRegionStats({});
      setData(res.regions);
    }
    load();
  }, []);

  const table = useReactTable({
    data,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(), // Enables instant browser sorting
  });

  return (
    <div className="border rounded-lg shadow bg-white overflow-hidden">
      <div className="p-4 border-b bg-gray-50">
        <h2 className="font-bold text-lg">Regional Summary (Client-Side)</h2>
      </div>
      <table className="w-full text-left text-sm">
        <thead className="bg-gray-100 border-b">
          {table.getHeaderGroups().map((headerGroup) => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <th
                  key={header.id}
                  className="p-3 font-semibold cursor-pointer hover:bg-gray-200 transition-colors select-none"
                  onClick={header.column.getToggleSortingHandler()}
                >
                   <div className="flex items-center gap-2">
                    {flexRender(header.column.columnDef.header, header.getContext())}
                    <ArrowUpDown className="w-4 h-4 text-gray-400" />
                   </div>
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody className="divide-y">
          {table.getRowModel().rows.map((row) => (
            <tr key={row.id} className="hover:bg-gray-50">
              {row.getVisibleCells().map((cell) => (
                <td key={cell.id} className="p-3">
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
