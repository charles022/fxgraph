"use client";
import { useState, useEffect } from "react";
import {
  useReactTable,
  getCoreRowModel,
  flexRender,
  PaginationState,
  SortingState,
} from "@tanstack/react-table";
import { analyticsClient } from "@/lib/client";
import { shipmentColumns } from "./columns";
import { ShipmentRow } from "@/gen/dashboard_pb";
import { ArrowUpDown } from "lucide-react"; // Assuming lucide-react is installed

export default function DashboardTable() {
  const [data, setData] = useState<ShipmentRow[]>([]);
  const [rowCount, setRowCount] = useState(0);
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  });
  const [sorting, setSorting] = useState<SortingState>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const fetchData = async () => {
      setLoading(true);
      try {
        const response = await analyticsClient.getShipments({
          pageNumber: pagination.pageIndex + 1,
          itemsPerPage: pagination.pageSize,
          sort: sorting.length > 0 ? {
               columnId: sorting[0].id, 
               isAscending: !sorting[0].desc 
          } : undefined,
          filters: [],
        });
        setData(response.rows);
        setRowCount(response.totalRowCount);
      } catch (err) {
        console.error("Failed to fetch shipments:", err);
      }
      setLoading(false);
    };

    fetchData();
  }, [pagination, sorting]);

  const table = useReactTable({
    data,
    columns: shipmentColumns,
    rowCount,
    state: { pagination, sorting },
    onPaginationChange: setPagination,
    onSortingChange: setSorting,
    manualPagination: true, // Server-side paging
    manualSorting: true,    // Server-side sorting
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <div className="border rounded-lg shadow bg-white overflow-hidden">
      <div className="p-4 border-b bg-gray-50 flex justify-between items-center">
        <h2 className="font-bold text-lg">Live Shipments (Server-Side)</h2>
        {loading && <span className="text-sm text-blue-600 animate-pulse">Updating...</span>}
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
            <tr key={row.id} className="hover:bg-gray-50 transition-colors">
              {row.getVisibleCells().map((cell) => (
                <td key={cell.id} className="p-3">
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      <div className="p-4 flex items-center justify-between bg-gray-50 border-t">
        <button
          className="px-3 py-1 border rounded bg-white disabled:opacity-50 hover:bg-gray-100"
          onClick={() => table.previousPage()}
          disabled={!table.getCanPreviousPage()}
        >
          Previous
        </button>
        <span className="text-sm text-gray-600">
          Page {table.getState().pagination.pageIndex + 1} of {table.getPageCount()}
        </span>
        <button
          className="px-3 py-1 border rounded bg-white disabled:opacity-50 hover:bg-gray-100"
          onClick={() => table.nextPage()}
          disabled={!table.getCanNextPage()}
        >
          Next
        </button>
      </div>
    </div>
  );
}
