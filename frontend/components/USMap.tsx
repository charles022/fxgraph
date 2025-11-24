"use client";

import React from "react";
import { ComposableMap, Geographies, Geography, Marker } from "react-simple-maps";
import { Location } from "@/gen/dashboard_pb";

const geoUrl = "https://cdn.jsdelivr.net/npm/us-atlas@3/states-10m.json";

interface USMapProps {
  locations: Location[];
  onLocationSelect: (id: string) => void;
  selectedId?: string | null;
}

export default function USMap({ locations, onLocationSelect, selectedId }: USMapProps) {
  return (
    <div className="w-full h-[500px] border rounded-lg shadow bg-slate-100 overflow-hidden relative">
       {/* Title/Legend Overlay */}
       <div className="absolute top-4 left-4 bg-white/80 backdrop-blur p-2 rounded shadow z-10 pointer-events-none">
          <h3 className="text-sm font-bold text-gray-700">US Facilities</h3>
          <p className="text-xs text-gray-500">Select a marker to view volume</p>
       </div>

      <ComposableMap projection="geoAlbersUsa" className="w-full h-full">
        <Geographies geography={geoUrl}>
          {({ geographies }) =>
            geographies.map((geo) => (
              <Geography
                key={geo.rsmKey}
                geography={geo}
                fill="#D1D5DB"
                stroke="#FFFFFF"
                strokeWidth={0.5}
                style={{
                    default: { outline: "none" },
                    hover: { outline: "none", fill: "#9CA3AF" },
                    pressed: { outline: "none" },
                }}
              />
            ))
          }
        </Geographies>
        {locations.map(({ id, name, latitude, longitude }) => (
          <Marker key={id} coordinates={[longitude, latitude]}>
            <circle
              r={6}
              fill={selectedId === id ? "#DC2626" : "#EA580C"} 
              stroke="#FFFFFF"
              strokeWidth={2}
              className="cursor-pointer transition-all hover:scale-125"
              onClick={() => onLocationSelect(id)}
            />
            <text
              textAnchor="middle"
              y={-10}
              style={{ fontFamily: "sans-serif", fill: "#374151", fontSize: "10px", fontWeight: "600", pointerEvents: 'none' }}
            >
              {name}
            </text>
          </Marker>
        ))}
      </ComposableMap>
    </div>
  );
}
