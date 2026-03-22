import { useEffect } from "react";
import { createPortal } from "react-dom";
import { TEMPLATES, type TemplateKey } from "../../lib/templates";
import { NODE_CONFIG } from "../../types";

interface TemplatePickerModalProps {
	onSelect: (key: TemplateKey) => void;
	onCancel: () => void;
}

const TEMPLATE_PREVIEWS: Record<TemplateKey, { dots: string[] }> = {
	five_whys: {
		dots: ["claim", "evidence", "evidence", "evidence", "evidence", "evidence"],
	},
	pro_con: {
		dots: ["claim", "evidence", "evidence", "rebuttal", "rebuttal"],
	},
	mece: {
		dots: [
			"claim",
			"claim",
			"claim",
			"claim",
			"evidence",
			"evidence",
			"evidence",
		],
	},
};

export default function TemplatePickerModal({
	onSelect,
	onCancel,
}: TemplatePickerModalProps) {
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
			<div className="absolute inset-0 bg-black/50" />
			<div
				className="relative z-10 w-96 rounded-lg border border-zinc-700 bg-zinc-900 p-5 shadow-xl"
				onClick={(e) => e.stopPropagation()}
			>
				<h3 className="mb-4 text-center text-sm font-semibold text-zinc-300">
					New from Template
				</h3>
				<div className="flex flex-col gap-3">
					{(
						Object.entries(TEMPLATES) as [
							TemplateKey,
							(typeof TEMPLATES)[TemplateKey],
						][]
					).map(([key, template]) => (
						<button
							key={key}
							onClick={() => onSelect(key)}
							className="flex flex-col gap-1.5 rounded-lg border border-zinc-700 p-3 text-left transition-colors hover:border-zinc-500 hover:bg-zinc-800"
						>
							<span className="text-sm font-medium text-zinc-200">
								{template.name}
							</span>
							<span className="text-xs text-zinc-500">
								{template.description}
							</span>
							<div className="mt-1 flex gap-1">
								{TEMPLATE_PREVIEWS[key].dots.map((type, i) => (
									<span
										key={i}
										className="inline-block h-2 w-2 rounded-full"
										style={{
											backgroundColor:
												NODE_CONFIG[type as keyof typeof NODE_CONFIG].border,
										}}
									/>
								))}
							</div>
						</button>
					))}
				</div>
			</div>
		</div>,
		document.body,
	);
}
