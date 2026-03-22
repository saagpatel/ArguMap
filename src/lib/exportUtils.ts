import { toPng } from "html-to-image";
import type { ArgFlowEdge, ArgFlowNode } from "../types";
import { EDGE_COLORS, STRENGTH_COLORS } from "../types";
import { tauriApi } from "./tauri";

export async function exportAsPng(mapTitle: string): Promise<void> {
	const viewport = document.querySelector(
		".react-flow__viewport",
	) as HTMLElement;
	if (!viewport) return;

	const dataUrl = await toPng(viewport, {
		backgroundColor: "#0A0A0A",
		pixelRatio: 2,
	});

	const link = document.createElement("a");
	link.href = dataUrl;
	link.download = `${mapTitle}.png`;
	link.click();
}

export async function exportAsJson(
	mapId: string,
	mapTitle: string,
): Promise<void> {
	const json = await tauriApi.exportMapJson(mapId);
	const blob = new Blob([json], { type: "application/json" });
	const url = URL.createObjectURL(blob);

	const link = document.createElement("a");
	link.href = url;
	link.download = `${mapTitle}.json`;
	link.click();
	URL.revokeObjectURL(url);
}

function escapeHtml(text: string): string {
	return text
		.replace(/&/g, "&amp;")
		.replace(/</g, "&lt;")
		.replace(/>/g, "&gt;")
		.replace(/"/g, "&quot;");
}

function getHandlePosition(
	node: { x: number; y: number; width: number; height: number },
	handleId: string | undefined,
): { x: number; y: number } {
	const w = node.width;
	const h = node.height;
	switch (handleId) {
		case "top":
			return { x: node.x + w / 2, y: node.y };
		case "bottom":
			return { x: node.x + w / 2, y: node.y + h };
		case "left":
			return { x: node.x, y: node.y + h / 2 };
		case "right":
			return { x: node.x + w, y: node.y + h / 2 };
		default:
			return { x: node.x + w / 2, y: node.y + h };
	}
}

export function exportAsHtml(
	nodes: ArgFlowNode[],
	edges: ArgFlowEdge[],
	mapTitle: string,
): void {
	// Filter out hidden nodes
	const visibleNodes = nodes.filter((n) => !n.hidden);
	const visibleNodeIds = new Set(visibleNodes.map((n) => n.id));
	const visibleEdges = edges.filter(
		(e) => visibleNodeIds.has(e.source) && visibleNodeIds.has(e.target),
	);

	// Compute bounding box
	const padding = 80;
	let minX = Infinity,
		minY = Infinity,
		maxX = -Infinity,
		maxY = -Infinity;
	for (const node of visibleNodes) {
		const w = node.measured?.width ?? node.data.width ?? 220;
		const h = node.measured?.height ?? node.data.height ?? 80;
		minX = Math.min(minX, node.position.x);
		minY = Math.min(minY, node.position.y);
		maxX = Math.max(maxX, node.position.x + w);
		maxY = Math.max(maxY, node.position.y + h);
	}
	if (!isFinite(minX)) {
		minX = 0;
		minY = 0;
		maxX = 800;
		maxY = 600;
	}
	const viewWidth = maxX - minX + padding * 2;
	const viewHeight = maxY - minY + padding * 2;
	const offsetX = minX - padding;
	const offsetY = minY - padding;

	// Build node lookup for edge rendering
	const nodeMap = new Map(
		visibleNodes.map((n) => [
			n.id,
			{
				x: n.position.x - offsetX,
				y: n.position.y - offsetY,
				width: n.measured?.width ?? n.data.width ?? 220,
				height: n.measured?.height ?? n.data.height ?? 80,
			},
		]),
	);

	// Generate edge SVG paths
	const edgePaths = visibleEdges
		.map((e) => {
			const sourceNode = nodeMap.get(e.source);
			const targetNode = nodeMap.get(e.target);
			if (!sourceNode || !targetNode) return "";

			const start = getHandlePosition(sourceNode, e.sourceHandle ?? "bottom");
			const end = getHandlePosition(targetNode, e.targetHandle ?? "top");
			const dy = Math.abs(end.y - start.y);
			const curve = Math.max(dy * 0.4, 50);

			// Determine control point direction based on handles
			const isVertical =
				(e.sourceHandle ?? "bottom") === "bottom" ||
				(e.sourceHandle ?? "bottom") === "top";
			let path: string;
			if (isVertical) {
				const dir = end.y > start.y ? 1 : -1;
				path = `M ${start.x} ${start.y} C ${start.x} ${start.y + curve * dir}, ${end.x} ${end.y - curve * dir}, ${end.x} ${end.y}`;
			} else {
				const dx = Math.abs(end.x - start.x);
				const hCurve = Math.max(dx * 0.4, 50);
				const dir = end.x > start.x ? 1 : -1;
				path = `M ${start.x} ${start.y} C ${start.x + hCurve * dir} ${start.y}, ${end.x - hCurve * dir} ${end.y}, ${end.x} ${end.y}`;
			}

			const color = EDGE_COLORS[e.data?.edge_type ?? "supports"];
			return `<path d="${path}" stroke="${color}" stroke-width="2" fill="none" />`;
		})
		.join("\n      ");

	const nodeConfig: Record<
		string,
		{ border: string; bg: string; label: string }
	> = {
		claim: { border: "#3B82F6", bg: "#1E3A5F", label: "Claim" },
		evidence: { border: "#10B981", bg: "#0F3028", label: "Evidence" },
		rebuttal: { border: "#EF4444", bg: "#3B1212", label: "Rebuttal" },
		counter_rebuttal: {
			border: "#F97316",
			bg: "#3B1E0A",
			label: "Counter-Rebuttal",
		},
	};

	// Generate node HTML
	const nodeHtml = visibleNodes
		.map((n) => {
			const x = n.position.x - offsetX;
			const y = n.position.y - offsetY;
			const w = n.measured?.width ?? n.data.width ?? 220;
			const config = nodeConfig[n.data.node_type] ?? nodeConfig.claim;
			const sourceHtml =
				n.data.node_type === "evidence" && n.data.source
					? `<div style="margin-top:8px;padding-top:8px;border-top:1px solid #374151;font-size:11px;color:#9CA3AF;">${escapeHtml(n.data.source)}</div>`
					: "";
			const strengthHtml =
				n.data.strength != null
					? `<div style="margin-top:6px;height:4px;border-radius:2px;background:${STRENGTH_COLORS[n.data.strength]};width:${(n.data.strength / 5) * 100}%;"></div>`
					: "";

			return `    <div style="position:absolute;left:${x}px;top:${y}px;width:${w}px;border:2px solid ${config.border};background:${config.bg};border-radius:8px;padding:28px 12px 12px;">
      <span style="position:absolute;left:8px;top:4px;background:${config.border};color:white;font-size:10px;font-weight:600;padding:2px 6px;border-radius:4px;text-transform:uppercase;letter-spacing:0.05em;">${config.label}</span>
      <div style="color:#E5E7EB;font-size:14px;">${escapeHtml(n.data.content)}</div>${sourceHtml}${strengthHtml}
    </div>`;
		})
		.join("\n");

	const html = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>${escapeHtml(mapTitle)} — ArguMap</title>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { background: #0F0F0F; color: #E5E7EB; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }
  header { padding: 16px 24px; border-bottom: 1px solid #222; background: #111; }
  header h1 { font-size: 16px; font-weight: 300; letter-spacing: 0.05em; color: #D1D5DB; }
  .canvas { position: relative; margin: 24px; }
  .legend { display: flex; gap: 16px; padding: 16px 24px; border-top: 1px solid #222; }
  .legend-item { display: flex; align-items: center; gap: 6px; font-size: 12px; color: #9CA3AF; }
  .legend-dot { width: 10px; height: 10px; border-radius: 50%; }
  footer { padding: 12px 24px; text-align: center; font-size: 11px; color: #4B5563; }
</style>
</head>
<body>
<header><h1>${escapeHtml(mapTitle)}</h1></header>
<div class="canvas" style="width:${viewWidth}px;height:${viewHeight}px;">
  <svg style="position:absolute;inset:0;width:100%;height:100%;pointer-events:none;">
    ${edgePaths}
  </svg>
${nodeHtml}
</div>
<div class="legend">
  <div class="legend-item"><span class="legend-dot" style="background:#3B82F6;"></span>Claim</div>
  <div class="legend-item"><span class="legend-dot" style="background:#10B981;"></span>Evidence</div>
  <div class="legend-item"><span class="legend-dot" style="background:#EF4444;"></span>Rebuttal</div>
  <div class="legend-item"><span class="legend-dot" style="background:#F97316;"></span>Counter-Rebuttal</div>
</div>
<footer>Exported from ArguMap Studio</footer>
</body>
</html>`;

	const blob = new Blob([html], { type: "text/html" });
	const url = URL.createObjectURL(blob);
	const link = document.createElement("a");
	link.href = url;
	link.download = `${mapTitle}.html`;
	link.click();
	URL.revokeObjectURL(url);
}
