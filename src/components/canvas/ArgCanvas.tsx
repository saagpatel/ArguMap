import "@xyflow/react/dist/style.css";

import {
	Background,
	BackgroundVariant,
	type Connection,
	Controls,
	MiniMap,
	type NodeChange,
	ReactFlow,
	useEdgesState,
	useNodesState,
	useReactFlow,
} from "@xyflow/react";
import { useCallback, useEffect, useRef, useState } from "react";
import { v4 as uuidv4 } from "uuid";
import { useCollapse } from "../../hooks/useCollapse";
import { useKeyboardShortcuts } from "../../hooks/useKeyboardShortcuts";
import { useMapLoader } from "../../hooks/useMapLoader";
import { useMapSync } from "../../hooks/useMapSync";
import { useUndo } from "../../hooks/useUndo";
import { exportAsHtml, exportAsPng } from "../../lib/exportUtils";
import { createNode } from "../../lib/nodeFactory";
import type { TemplateKey } from "../../lib/templates";
import type {
	ArgFlowEdge,
	ArgFlowNode,
	ArgMap,
	ArgNodeData,
	EdgeType,
	NodeType,
} from "../../types";
import Sidebar from "../sidebar/Sidebar";
import EdgeTypeModal from "./EdgeTypeModal";
import EmptyState from "./EmptyState";
import { edgeTypes } from "./edges";
import { nodeTypes } from "./nodes";

interface ArgCanvasProps {
	mapId: string | null;
	mapTitle: string;
	maps: ArgMap[];
	activeMapId: string | null;
	onSelectMap: (mapId: string) => void;
	onCreateMap: () => void;
	onCreateFromTemplate: (key: TemplateKey) => void;
	onRenameMap: (mapId: string, title: string) => void;
	onDeleteMap: (mapId: string) => void;
}

export default function ArgCanvas({
	mapId,
	mapTitle,
	maps,
	activeMapId,
	onSelectMap,
	onCreateMap,
	onCreateFromTemplate,
	onRenameMap,
	onDeleteMap,
}: ArgCanvasProps) {
	const [nodes, setNodes, onNodesChange] = useNodesState<ArgFlowNode>([]);
	const [edges, setEdges, onEdgesChange] = useEdgesState<ArgFlowEdge>([]);
	const [pendingConnection, setPendingConnection] = useState<Connection | null>(
		null,
	);
	const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);

	const { screenToFlowPosition, fitView, getNodes, getEdges } = useReactFlow<
		ArgFlowNode,
		ArgFlowEdge
	>();

	// Sync hooks
	const { isHydrating, syncOnPosition, syncOnContent, syncImmediate } =
		useMapSync(mapId);

	// Undo hook
	const {
		recordAddNode,
		recordDeleteNodes,
		recordAddEdge,
		recordDeleteEdge,
		undo,
		clear: clearUndo,
	} = useUndo(setNodes, setEdges, syncImmediate);

	// Collapse/expand hook
	const {
		collapsedNodeIds,
		hiddenNodeIds,
		toggleCollapse,
		getDescendantCount,
		hasDescendants,
		clear: clearCollapse,
	} = useCollapse(edges);

	// Clear undo + collapse when map changes
	useEffect(() => {
		clearUndo();
		clearCollapse();
	}, [mapId, clearUndo, clearCollapse]);

	// Apply hidden state from collapse
	useEffect(() => {
		setNodes((nds) =>
			nds.map((n) => ({
				...n,
				hidden: hiddenNodeIds.has(n.id),
				data: {
					...n.data,
					isCollapsed: collapsedNodeIds.has(n.id),
					hiddenDescendantCount: collapsedNodeIds.has(n.id)
						? getDescendantCount(n.id)
						: 0,
					onToggleCollapse: hasDescendants(n.id) ? toggleCollapse : undefined,
				},
			})),
		);
	}, [
		hiddenNodeIds,
		collapsedNodeIds,
		setNodes,
		toggleCollapse,
		getDescendantCount,
		hasDescendants,
	]);

	// Stable onUpdate via ref pattern — identity never changes, avoids node re-renders
	// Also handles type switching: updates node.type when node_type changes
	const handleNodeUpdateRef = useRef<ArgNodeData["onUpdate"]>(() => {});
	handleNodeUpdateRef.current = (id, updates) => {
		setNodes((nds) =>
			nds.map((n) => {
				if (n.id !== id) return n;
				const newData = { ...n.data, ...updates };
				return {
					...n,
					data: newData,
					...(updates.node_type ? { type: updates.node_type } : {}),
				};
			}),
		);
		syncOnContent();
	};
	const handleNodeUpdate = useCallback<ArgNodeData["onUpdate"]>(
		(id, updates) => handleNodeUpdateRef.current(id, updates),
		[],
	);

	// Hydrate from SQLite on map switch
	useMapLoader(mapId, isHydrating, handleNodeUpdate);

	// Route position/dimension changes to debounced sync
	const handleNodesChange = useCallback(
		(changes: NodeChange<ArgFlowNode>[]) => {
			onNodesChange(changes);
			if (isHydrating.current) return;
			if (
				changes.some((c) => c.type === "position" || c.type === "dimensions")
			) {
				syncOnPosition();
			}
		},
		[onNodesChange, syncOnPosition, isHydrating],
	);

	// Selection tracking for NodeEditor
	const handleSelectionChange = useCallback(
		({
			nodes: selectedNodes,
		}: {
			nodes: ArgFlowNode[];
			edges: ArgFlowEdge[];
		}) => {
			setSelectedNodeId(
				selectedNodes.length === 1 ? selectedNodes[0].id : null,
			);
		},
		[],
	);

	// Deletion: stash data in onBeforeDelete, record + sync in onDelete
	const pendingDeleteRef = useRef<{
		nodes: ArgFlowNode[];
		edges: ArgFlowEdge[];
	} | null>(null);

	const handleBeforeDelete = useCallback(
		async ({
			nodes: delNodes,
			edges: delEdges,
		}: {
			nodes: ArgFlowNode[];
			edges: ArgFlowEdge[];
		}) => {
			if (delNodes.length > 0 && delEdges.length > 0) {
				const confirmed = window.confirm(
					`Delete this node? This will also remove ${delEdges.length} connected edge(s).`,
				);
				if (!confirmed) return false;
			}
			pendingDeleteRef.current = { nodes: delNodes, edges: delEdges };
			return true;
		},
		[],
	);

	const handleDelete = useCallback(() => {
		if (pendingDeleteRef.current) {
			const { nodes: delNodes, edges: delEdges } = pendingDeleteRef.current;
			if (delNodes.length > 0) {
				recordDeleteNodes(delNodes, delEdges);
			} else if (delEdges.length > 0) {
				recordDeleteEdge(delEdges[0]);
			}
			pendingDeleteRef.current = null;
		}
		syncImmediate();
	}, [recordDeleteNodes, recordDeleteEdge, syncImmediate]);

	// Add node at viewport center
	const handleAddNode = useCallback(
		(type: NodeType) => {
			const container = document.querySelector(".react-flow");
			const rect = container?.getBoundingClientRect();
			const center = screenToFlowPosition({
				x: (rect?.left ?? 0) + (rect?.width ?? 800) / 2,
				y: (rect?.top ?? 0) + (rect?.height ?? 600) / 2,
			});
			const node = createNode(type, center, handleNodeUpdate);
			setNodes((nds) => [
				...nds.map((n) => ({ ...n, selected: false })),
				{ ...node, selected: true },
			]);
			recordAddNode(node.id);
			syncImmediate();
		},
		[
			screenToFlowPosition,
			handleNodeUpdate,
			setNodes,
			recordAddNode,
			syncImmediate,
		],
	);

	// --- Edge creation flow ---

	const handleConnect = useCallback((connection: Connection) => {
		setPendingConnection(connection);
	}, []);

	const handleEdgeTypeConfirm = useCallback(
		(edgeType: EdgeType) => {
			if (!pendingConnection) return;

			const isDuplicate = edges.some(
				(e) =>
					e.source === pendingConnection.source &&
					e.target === pendingConnection.target &&
					e.data?.edge_type === edgeType,
			);
			if (isDuplicate) {
				setPendingConnection(null);
				return;
			}

			const newEdge: ArgFlowEdge = {
				id: uuidv4(),
				source: pendingConnection.source,
				target: pendingConnection.target,
				sourceHandle: pendingConnection.sourceHandle ?? undefined,
				targetHandle: pendingConnection.targetHandle ?? undefined,
				data: { edge_type: edgeType },
			};
			setEdges((eds) => [...eds, newEdge]);
			recordAddEdge(newEdge.id);
			setPendingConnection(null);
			syncImmediate();
		},
		[pendingConnection, edges, setEdges, recordAddEdge, syncImmediate],
	);

	const handleEdgeTypeCancel = useCallback(() => {
		setPendingConnection(null);
	}, []);

	const isValidConnection = useCallback(
		(connection: Connection | ArgFlowEdge) =>
			connection.source !== connection.target,
		[],
	);

	// --- Export ---

	const handleExportPng = useCallback(async () => {
		if (!mapId) return;
		await fitView({ padding: 0.1, duration: 0 });
		await new Promise((resolve) => setTimeout(resolve, 300));
		await exportAsPng(mapTitle);
	}, [mapId, fitView, mapTitle]);

	const handleExportHtml = useCallback(() => {
		exportAsHtml(getNodes(), getEdges(), mapTitle);
	}, [getNodes, getEdges, mapTitle]);

	// Keyboard shortcuts
	useKeyboardShortcuts(handleAddNode, undo, handleExportPng, handleExportHtml);

	// Derive selected node for sidebar
	const selectedNode =
		selectedNodeId !== null
			? (nodes.find((n) => n.id === selectedNodeId) ?? null)
			: null;

	return (
		<div className="flex h-full">
			<aside className="w-56 shrink-0 border-r border-zinc-800 bg-[#111111]">
				<Sidebar
					maps={maps}
					activeMapId={activeMapId}
					onSelectMap={onSelectMap}
					onCreateMap={onCreateMap}
					onCreateFromTemplate={onCreateFromTemplate}
					onRenameMap={onRenameMap}
					onDeleteMap={onDeleteMap}
					onAddNode={handleAddNode}
					selectedNode={selectedNode}
					onUpdateNode={handleNodeUpdate}
				/>
			</aside>
			<div className="relative flex-1">
				{nodes.length === 0 && !isHydrating.current && <EmptyState />}
				<ReactFlow
					nodes={nodes}
					edges={edges}
					onNodesChange={handleNodesChange}
					onEdgesChange={onEdgesChange}
					onConnect={handleConnect}
					onSelectionChange={handleSelectionChange}
					onBeforeDelete={handleBeforeDelete}
					onDelete={handleDelete}
					isValidConnection={isValidConnection}
					nodeTypes={nodeTypes}
					edgeTypes={edgeTypes}
					connectionLineStyle={{ stroke: "#555", strokeWidth: 2 }}
					fitView
					proOptions={{ hideAttribution: true }}
					style={{ backgroundColor: "#0F0F0F" }}
				>
					<Background variant={BackgroundVariant.Dots} color="#333" gap={20} />
					<MiniMap
						nodeColor={() => "#555"}
						maskColor="rgba(0,0,0,0.7)"
						style={{ background: "#1a1a1a" }}
					/>
					<Controls />
				</ReactFlow>
			</div>
			{pendingConnection && (
				<EdgeTypeModal
					onConfirm={handleEdgeTypeConfirm}
					onCancel={handleEdgeTypeCancel}
				/>
			)}
		</div>
	);
}
