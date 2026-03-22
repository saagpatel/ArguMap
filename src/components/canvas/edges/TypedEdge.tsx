import {
	BaseEdge,
	EdgeLabelRenderer,
	type EdgeProps,
	getBezierPath,
} from "@xyflow/react";
import { useState } from "react";
import { type ArgFlowEdge, EDGE_COLORS } from "../../../types";

export default function TypedEdge({
	id,
	sourceX,
	sourceY,
	sourcePosition,
	targetX,
	targetY,
	targetPosition,
	data,
	selected,
	markerEnd,
}: EdgeProps<ArgFlowEdge>) {
	const [hovered, setHovered] = useState(false);
	const [edgePath, labelX, labelY] = getBezierPath({
		sourceX,
		sourceY,
		sourcePosition,
		targetX,
		targetY,
		targetPosition,
	});

	const edgeType = data?.edge_type ?? "supports";
	const color = EDGE_COLORS[edgeType];

	return (
		<>
			{/* Invisible wide path for hover detection — 2px edges are impossible to hover */}
			<path
				d={edgePath}
				fill="none"
				stroke="transparent"
				strokeWidth={20}
				onMouseEnter={() => setHovered(true)}
				onMouseLeave={() => setHovered(false)}
			/>
			<BaseEdge
				id={id}
				path={edgePath}
				markerEnd={markerEnd}
				style={{
					stroke: color,
					strokeWidth: 2,
					strokeDasharray: hovered ? "5 5" : "none",
					animation: hovered ? "edge-dash 0.5s linear infinite" : "none",
				}}
			/>
			{(hovered || selected) && (
				<EdgeLabelRenderer>
					<div
						style={{
							position: "absolute",
							transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
							pointerEvents: "none",
						}}
						className="rounded bg-zinc-800/90 px-2 py-0.5 text-[11px] text-zinc-300"
					>
						{edgeType.replace("_", " ")}
					</div>
				</EdgeLabelRenderer>
			)}
		</>
	);
}
