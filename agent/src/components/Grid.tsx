import type { Page, Button } from "../types/protocol";

interface Props {
  page: Page;
  selected: { row: number; col: number } | null;
  onSelect: (row: number, col: number) => void;
}

export function Grid({ page, selected, onSelect }: Props) {
  const cells: Array<{ row: number; col: number; button: Button | undefined }> = [];
  for (let row = 0; row < page.rows; row++) {
    for (let col = 0; col < page.cols; col++) {
      const button = page.buttons.find((b) => b.row === row && b.col === col);
      cells.push({ row, col, button });
    }
  }

  return (
    <div
      className="grid"
      style={{
        gridTemplateColumns: `repeat(${page.cols}, 1fr)`,
        gridTemplateRows: `repeat(${page.rows}, 1fr)`,
      }}
    >
      {cells.map(({ row, col, button }) => {
        const isSelected = selected?.row === row && selected?.col === col;
        return (
          <button
            key={`${row}-${col}`}
            className={`grid-cell ${button ? "filled" : "empty"} ${
              isSelected ? "selected" : ""
            }`}
            onClick={() => onSelect(row, col)}
          >
            {button ? (
              <>
                {button.iconBase64 && (
                  <img
                    className="grid-cell__icon"
                    src={`data:image/png;base64,${button.iconBase64}`}
                    alt=""
                  />
                )}
                <span className="grid-cell__label">{button.label ?? "—"}</span>
              </>
            ) : (
              <span className="grid-cell__plus">+</span>
            )}
          </button>
        );
      })}
    </div>
  );
}
