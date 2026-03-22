import { useEffect } from "react";
import { createPortal } from "react-dom";
import { EDGE_COLORS, type EdgeType } from "../../types";

const EDGE_TYPE_OPTIONS: { type: EdgeType; label: string }[] = [
	{ type: "supports", label: "Supports" },
	{ type: "rebuts", label: "Rebuts" },
	{ type: "qualifies", label: "Qualifies" },
	{ type: "depends_on", label: "Depends On" },
];

interface EdgeTypeModalProps {
	onConfirm: (type: EdgeType) => void;
	onCancel: () => void;
}

export default function EdgeTypeModal({
	onConfirm,
	onCancel,
}: EdgeTypeModalProps) {
	// Escape key closes modal
	useEffect(() => {
		const handler = (e: KeyboardEvent) => {
			if (e.key === "Escape") onCancel();
		};
		document.addEventListener("keydown", handler);
		return () => document.removeEventListener("keydown", handler);
	}, [onCancel]);

	return createPortal(
		<div
			className="fixed inset-0 z-50 flex items-center justify-center"
			onClick={onCancel}
		>
			{/* Backdrop */}
			<div className="absolute inset-0 bg-black/50" />

			{/* Modal card */}
			<div
				className="relative z-10 rounded-lg border border-zinc-700 bg-zinc-900 p-4 shadow-xl"
				onClick={(e) => e.stopPropagation()}
			>
				<h3 className="mb-3 text-center text-sm font-semibold text-zinc-300">
					Select Edge Type
				</h3>
				<div className="flex flex-col gap-2">
					{EDGE_TYPE_OPTIONS.map(({ type, label }) => (
						<button
							key={type}
							onClick={() => onConfirm(type)}
							className="flex items-center gap-3 rounded-md px-4 py-2 text-sm text-zinc-200 transition-colors hover:bg-zinc-800"
						>
							<span
								className="inline-block h-3 w-3 rounded-full"
								style={{ backgroundColor: EDGE_COLORS[type] }}
							/>
							{label}
						</button>
					))}
				</div>
			</div>
		</div>,
		document.body,
	);
}
