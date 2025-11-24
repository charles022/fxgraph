"use client";

import { useState, useEffect } from "react";
import { analyticsClient } from "@/lib/client";
import { Location, FacilityStats } from "@/gen/dashboard_pb";
import USMap from "./USMap";
import VolumeTable from "./VolumeTable";

export default function MapDashboard() {
  const [locations, setLocations] = useState<Location[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [stats, setStats] = useState<FacilityStats | null>(null);
  const [loadingStats, setLoadingStats] = useState(false);

  useEffect(() => {
    async function loadLocations() {
      try {
        const res = await analyticsClient.getLocations({});
        setLocations(res.locations);
      } catch (e) {
        console.error("Failed to load locations", e);
      }
    }
    loadLocations();
  }, []);

  useEffect(() => {
    if (!selectedId) {
        setStats(null);
        return;
    }
    async function loadStats() {
      setLoadingStats(true);
      try {
        console.log(`Fetching stats for facility: ${selectedId} from server...`);
        // Pattern B: Triggers server-side request
        const res = await analyticsClient.getFacilityStats({ facilityId: selectedId ?? "" });
        setStats(res);
      } catch (e) {
        console.error("Failed to load stats", e);
      } finally {
        setLoadingStats(false);
      }
    }
    loadStats();
  }, [selectedId]);

  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <div className="lg:col-span-2">
        <USMap 
            locations={locations} 
            onLocationSelect={setSelectedId} 
            selectedId={selectedId} 
        />
      </div>
      <div className="lg:col-span-1 h-[500px]">
        <VolumeTable stats={stats} isLoading={loadingStats} />
      </div>
    </div>
  );
}
