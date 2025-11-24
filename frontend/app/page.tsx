"use client";

import { useState } from "react";
import DashboardTable from "@/components/DashboardTable";
import RegionTable from "@/components/RegionTable";
import MapDashboard from "@/components/MapDashboard";
import { Tabs } from "@/components/Tabs";

export default function Page() {
  const [activeTab, setActiveTab] = useState("dashboard");

  return (
    <main className="min-h-screen bg-gray-50 p-8">
      <div className="max-w-7xl mx-auto space-y-8">
        
        <header className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Logistics Analytics PoC</h1>
          <p className="text-gray-600 mt-2">
            Demonstrating Hybrid Data Architecture: 
            <span className="font-semibold text-blue-600"> Client-Side Aggregates</span> vs. 
            <span className="font-semibold text-green-600"> Server-Side Granularity</span>.
          </p>
        </header>

        <Tabs 
            tabs={[
                { label: "Dashboard", value: "dashboard" },
                { label: "Facility Map", value: "map" }
            ]}
            activeTab={activeTab}
            onTabChange={setActiveTab}
        />

        {activeTab === "dashboard" && (
            <div className="grid grid-cols-1 gap-8 max-w-5xl">
            {/* Pattern B: Small Data */}
            <section>
                <div className="mb-2 flex items-center gap-2">
                <span className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded font-bold">PATTERN B</span>
                <p className="text-sm text-gray-500">Small dataset fetched once. Instant sorting in browser.</p>
                </div>
                <RegionTable />
            </section>

            {/* Pattern A: Big Data */}
            <section>
                <div className="mb-2 flex items-center gap-2">
                <span className="px-2 py-1 bg-green-100 text-green-800 text-xs rounded font-bold">PATTERN A</span>
                <p className="text-sm text-gray-500">Massive dataset (10k rows). Sorting triggers server request.</p>
                </div>
                <DashboardTable />
            </section>
            </div>
        )}

        {activeTab === "map" && (
            <section>
                 <div className="mb-4 flex items-center gap-2">
                    <span className="px-2 py-1 bg-purple-100 text-purple-800 text-xs rounded font-bold">MAP VIEW</span>
                    <p className="text-sm text-gray-500">Geospatial Analysis with Server-Side Drilldown.</p>
                 </div>
                 <MapDashboard />
            </section>
        )}

      </div>
    </main>
  );
}