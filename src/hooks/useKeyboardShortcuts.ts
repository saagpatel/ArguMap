import { useEffect } from "react";
import type { NodeType } from "../types";

export function useKeyboardShortcuts(
	onAddNode: (type: NodeType) => void,
	onUndo: () => void,
	onExport: () => void,
) {
	useEffect(() => {
		const handler = (e: KeyboardEvent) => {
			// Suppress when typing in input fields
			const target = e.target as HTMLElement;
			if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") return;

			// Cmd/Ctrl shortcuts
			if (e.metaKey || e.ctrlKey) {
				switch (e.key.toLowerCase()) {
					case "z":
						e.preventDefault();
						onUndo();
						return;
					case "e":
						e.preventDefault();
						onExport();
						return;
				}
				return;
			}

			// Don't interfere with other modifier combos
			if (e.altKey) return;

			switch (e.key.toLowerCase()) {
				case "c":
					e.preventDefault();
					onAddNode("claim");
					break;
				case "e":
					e.preventDefault();
					onAddNode("evidence");
					break;
				case "r":
					e.preventDefault();
					onAddNode(e.shiftKey ? "counter_rebuttal" : "rebuttal");
					break;
			}
		};

		document.addEventListener("keydown", handler);
		return () => document.removeEventListener("keydown", handler);
	}, [onAddNode, onUndo, onExport]);
}
