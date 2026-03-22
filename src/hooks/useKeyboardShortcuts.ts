import { useEffect } from "react";
import type { NodeType } from "../types";

export function useKeyboardShortcuts(
	onAddNode: (type: NodeType) => void,
	onUndo: () => void,
	onExportPng: () => void,
	onExportHtml: () => void,
) {
	useEffect(() => {
		const handler = (e: KeyboardEvent) => {
			const target = e.target as HTMLElement;
			if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") return;

			if (e.metaKey || e.ctrlKey) {
				switch (e.key.toLowerCase()) {
					case "z":
						e.preventDefault();
						onUndo();
						return;
					case "e":
						e.preventDefault();
						if (e.shiftKey) {
							onExportHtml();
						} else {
							onExportPng();
						}
						return;
				}
				return;
			}

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
	}, [onAddNode, onUndo, onExportPng, onExportHtml]);
}
