"use client";

import { FacilityStats } from "@/gen/dashboard_pb";

interface VolumeTableProps {
  stats?: FacilityStats | null;
  isLoading: boolean;
}

export default function VolumeTable({ stats, isLoading }: VolumeTableProps) {
  if (isLoading) {
    return (
      <div className="border rounded-lg shadow bg-white p-8 text-center text-gray-500 h-full flex items-center justify-center">
        Loading...
      </div>
    );
  }

  if (!stats) {
    return (
      <div className="border rounded-lg shadow bg-white p-8 text-center text-gray-500 h-full flex items-center justify-center">
        Select a facility on the map to view volume data.
      </div>
    );
  }

  const days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

  return (
    <div className="border rounded-lg shadow bg-white overflow-hidden h-full">
      <div className="p-4 border-b bg-gray-50">
        <h2 className="font-bold text-lg">Facility Volume: {stats.facilityId}</h2>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm">
          <thead className="bg-gray-100 border-b">
            <tr>
              <th className="p-3 font-semibold border-r text-gray-600">Week</th>
              {days.map((day) => (
                <th key={day} className="p-3 font-semibold text-gray-600">
                  {day}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y">
            {stats.weeks.map((week, i) => (
              <tr key={i} className="hover:bg-gray-50">
                <td className="p-3 font-medium border-r bg-gray-50">Week {week.weekNumber}</td>
                {week.dailyVolumes.map((vol, j) => (
                  <td key={j} className="p-3">
                    {vol.toLocaleString()}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
