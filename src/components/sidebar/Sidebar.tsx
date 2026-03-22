import type {
	ArgFlowNode,
	ArgMap,
	ArgNodeData,
	NodeType,
	TemplateKey,
} from "../../types";
import AddNodePanel from "./AddNodePanel";
import MapLibrary from "./MapLibrary";
import NodeEditor from "./NodeEditor";

interface SidebarProps {
	maps: ArgMap[];
	activeMapId: string | null;
	onSelectMap: (mapId: string) => void;
	onCreateMap: () => void;
	onCreateFromTemplate: (key: TemplateKey) => void;
	onRenameMap: (mapId: string, title: string) => void;
	onDeleteMap: (mapId: string) => void;
	onAddNode: (type: NodeType) => void;
	selectedNode: ArgFlowNode | null;
	onUpdateNode: ArgNodeData["onUpdate"];
}

export default function Sidebar({
	maps,
	activeMapId,
	onSelectMap,
	onCreateMap,
	onCreateFromTemplate,
	onRenameMap,
	onDeleteMap,
	onAddNode,
	selectedNode,
	onUpdateNode,
}: SidebarProps) {
	return (
		<div className="flex h-full flex-col">
			<div className="flex-1 overflow-y-auto p-3">
				<MapLibrary
					maps={maps}
					activeMapId={activeMapId}
					onSelectMap={onSelectMap}
					onCreateMap={onCreateMap}
					onCreateFromTemplate={onCreateFromTemplate}
					onRenameMap={onRenameMap}
					onDeleteMap={onDeleteMap}
				/>
			</div>
			<div className="border-t border-zinc-800 p-3">
				{selectedNode ? (
					<NodeEditor node={selectedNode} onUpdate={onUpdateNode} />
				) : (
					<AddNodePanel onAddNode={onAddNode} />
				)}
			</div>
		</div>
	);
}
