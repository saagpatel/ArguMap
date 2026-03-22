import { useEffect, useRef, useState } from "react";
import type { ArgMap } from "../../types";

interface MapLibraryProps {
	maps: ArgMap[];
	activeMapId: string | null;
	onSelectMap: (mapId: string) => void;
	onCreateMap: () => void;
	onRenameMap: (mapId: string, title: string) => void;
	onDeleteMap: (mapId: string) => void;
}

interface ContextMenuState {
	mapId: string;
	x: number;
	y: number;
}

export default function MapLibrary({
	maps,
	activeMapId,
	onSelectMap,
	onCreateMap,
	onRenameMap,
	onDeleteMap,
}: MapLibraryProps) {
	const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
	const [renamingMapId, setRenamingMapId] = useState<string | null>(null);
	const [renameValue, setRenameValue] = useState("");
	const renameInputRef = useRef<HTMLInputElement>(null);

	useEffect(() => {
		if (!contextMenu) return;

		const handleMouseDown = (e: MouseEvent) => {
			const target = e.target as HTMLElement;
			if (!target.closest(".context-menu")) {
				setContextMenu(null);
			}
		};

		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.key === "Escape") setContextMenu(null);
		};

		document.addEventListener("mousedown", handleMouseDown);
		document.addEventListener("keydown", handleKeyDown);
		return () => {
			document.removeEventListener("mousedown", handleMouseDown);
			document.removeEventListener("keydown", handleKeyDown);
		};
	}, [contextMenu]);

	useEffect(() => {
		if (renamingMapId && renameInputRef.current) {
			renameInputRef.current.focus();
			renameInputRef.current.select();
		}
	}, [renamingMapId]);

	const handleContextMenu = (e: React.MouseEvent, mapId: string) => {
		e.preventDefault();
		setContextMenu({ mapId, x: e.clientX, y: e.clientY });
	};

	const handleRenameStart = (mapId: string, currentTitle: string) => {
		setContextMenu(null);
		setRenamingMapId(mapId);
		setRenameValue(currentTitle);
	};

	const handleRenameCommit = (mapId: string) => {
		const trimmed = renameValue.trim();
		if (trimmed) {
			onRenameMap(mapId, trimmed);
		}
		setRenamingMapId(null);
	};

	const handleRenameKeyDown = (
		e: React.KeyboardEvent<HTMLInputElement>,
		mapId: string,
	) => {
		if (e.key === "Enter") {
			e.preventDefault();
			handleRenameCommit(mapId);
		} else if (e.key === "Escape") {
			setRenamingMapId(null);
		}
	};

	const handleDelete = (mapId: string, title: string) => {
		setContextMenu(null);
		if (window.confirm(`Delete "${title}"? This cannot be undone.`)) {
			onDeleteMap(mapId);
		}
	};

	return (
		<div className="flex flex-col gap-1">
			<div className="mb-1 flex items-center justify-between">
				<h3 className="text-xs font-semibold uppercase tracking-wider text-zinc-500">
					Maps
				</h3>
				<button
					onClick={onCreateMap}
					className="rounded px-1.5 py-0.5 text-xs text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-200"
				>
					+ New
				</button>
			</div>

			<div className="flex flex-col gap-0.5">
				{maps.map((map) => {
					const isActive = map.id === activeMapId;
					const isRenaming = renamingMapId === map.id;

					return (
						<div
							key={map.id}
							onClick={() => !isRenaming && onSelectMap(map.id)}
							onContextMenu={(e) => handleContextMenu(e, map.id)}
							className={[
								"flex cursor-pointer items-center rounded-md border-l-2 px-2 py-1.5 text-sm transition-colors",
								isActive
									? "border-blue-500 bg-zinc-800 text-zinc-100"
									: "border-transparent text-zinc-300 hover:bg-zinc-800",
							].join(" ")}
						>
							{isRenaming ? (
								<input
									ref={renameInputRef}
									value={renameValue}
									onChange={(e) => setRenameValue(e.target.value)}
									onBlur={() => handleRenameCommit(map.id)}
									onKeyDown={(e) => handleRenameKeyDown(e, map.id)}
									onClick={(e) => e.stopPropagation()}
									className="w-full bg-transparent text-sm text-zinc-100 outline-none"
								/>
							) : (
								<span className="truncate">{map.title}</span>
							)}
						</div>
					);
				})}
			</div>

			{contextMenu &&
				(() => {
					const map = maps.find((m) => m.id === contextMenu.mapId);
					if (!map) return null;
					return (
						<div
							className="context-menu"
							style={{
								position: "fixed",
								top: contextMenu.y,
								left: contextMenu.x,
								zIndex: 50,
							}}
						>
							<div className="flex flex-col overflow-hidden rounded-md border border-zinc-700 bg-zinc-900 shadow-lg">
								<button
									onClick={() => handleRenameStart(map.id, map.title)}
									className="px-4 py-2 text-left text-sm text-zinc-300 transition-colors hover:bg-zinc-800"
								>
									Rename
								</button>
								<button
									onClick={() => handleDelete(map.id, map.title)}
									className="px-4 py-2 text-left text-sm text-red-400 transition-colors hover:bg-zinc-800"
								>
									Delete
								</button>
							</div>
						</div>
					);
				})()}
		</div>
	);
}
