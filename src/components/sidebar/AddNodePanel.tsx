import { NODE_CONFIG, type NodeType } from "../../types";

interface AddNodePanelProps {
	onAddNode: (type: NodeType) => void;
}

const NODE_TYPES: NodeType[] = [
	"claim",
	"evidence",
	"rebuttal",
	"counter_rebuttal",
];

export default function AddNodePanel({ onAddNode }: AddNodePanelProps) {
	return (
		<div className="flex flex-col gap-2">
			<h3 className="mb-1 text-xs font-semibold uppercase tracking-wider text-zinc-500">
				Add Node
			</h3>
			{NODE_TYPES.map((type) => (
				<button
					key={type}
					onClick={() => onAddNode(type)}
					className="w-full rounded-md border-2 px-3 py-2 text-left text-sm text-zinc-200 transition-colors hover:brightness-125"
					style={{
						borderColor: NODE_CONFIG[type].border,
						backgroundColor: NODE_CONFIG[type].bg,
					}}
				>
					+ {NODE_CONFIG[type].label}
				</button>
			))}
		</div>
	);
}
