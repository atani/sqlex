import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { Terminal } from "./Terminal";
import { TerminalLine, TypingLine, TextSegment } from "./TerminalLine";

export const SqlexDemo: React.FC = () => {
  const frame = useCurrentFrame();

  // プロンプト
  const promptPrefix: TextSegment[] = [
    { text: "$ ", color: "green", bold: true },
  ];

  // === Phase 1: check コマンド（エラー） ===
  const checkCommand = "sqlex check query.sql";
  const checkOutputStart = 55;

  const checkOutputLines: { segments: TextSegment[]; delay: number }[] = [
    {
      segments: [
        { text: "✗ ", color: "red", bold: true },
        { text: "query.sql", color: "white", bold: true },
        { text: " - ", color: "gray" },
        { text: "1 error(s)", color: "red" },
      ],
      delay: 0,
    },
    { segments: [{ text: "" }], delay: 2 },
    {
      segments: [
        { text: "  Syntax error ", color: "red" },
        { text: "(line 4, col 6)", color: "gray" },
        { text: ": Expected expression, found: ", color: "white" },
        { text: "FROM", color: "cyan", bold: true },
      ],
      delay: 4,
    },
    {
      segments: [
        { text: "  💡 ", color: "yellow" },
        { text: "Line 3 may have a trailing comma that should be removed", color: "yellow" },
      ],
      delay: 8,
    },
    { segments: [{ text: "" }], delay: 10 },
    {
      segments: [
        { text: "  2 ", color: "gray" },
        { text: "│ ", color: "gray" },
        { text: "  name,", color: "white" },
      ],
      delay: 12,
    },
    {
      segments: [
        { text: "  3 ", color: "yellow", bold: true },
        { text: "│ ", color: "gray" },
        { text: "  email,", color: "white" },
        { text: "  ← check here", color: "yellow" },
      ],
      delay: 14,
    },
    {
      segments: [
        { text: "  4 ", color: "red", bold: true },
        { text: "│ ", color: "gray" },
        { text: "FROM", color: "cyan", bold: true },
        { text: " users", color: "white" },
      ],
      delay: 16,
    },
    {
      segments: [
        { text: "    ", color: "gray" },
        { text: "│ ", color: "gray" },
        { text: "     ", color: "white" },
        { text: "^", color: "red", bold: true },
      ],
      delay: 18,
    },
    {
      segments: [
        { text: "  5 ", color: "gray" },
        { text: "│ ", color: "gray" },
        { text: "WHERE active = 1", color: "white" },
      ],
      delay: 20,
    },
    { segments: [{ text: "" }], delay: 22 },
    {
      segments: [
        { text: "Total: ", color: "gray" },
        { text: "1", color: "white", bold: true },
        { text: " file(s), ", color: "gray" },
        { text: "1", color: "red", bold: true },
        { text: " error(s)", color: "gray" },
      ],
      delay: 24,
    },
  ];

  // === Phase 2: fix コマンド ===
  const fixCommandStart = 120;
  const fixCommand = "sqlex fix query.sql";
  const fixOutputStart = 165;

  const fixOutputLines: { segments: TextSegment[]; delay: number }[] = [
    {
      segments: [
        { text: "✓ ", color: "green", bold: true },
        { text: "query.sql", color: "white", bold: true },
        { text: " - ", color: "gray" },
        { text: "1 fix(es) applied", color: "green" },
      ],
      delay: 0,
    },
    { segments: [{ text: "" }], delay: 3 },
    {
      segments: [
        { text: "  Fixed: ", color: "green" },
        { text: "Removed trailing comma on line 3", color: "white" },
      ],
      delay: 6,
    },
    { segments: [{ text: "" }], delay: 9 },
    {
      segments: [
        { text: "Total: ", color: "gray" },
        { text: "1", color: "white", bold: true },
        { text: " file(s), ", color: "gray" },
        { text: "1", color: "green", bold: true },
        { text: " fix(es)", color: "gray" },
      ],
      delay: 12,
    },
  ];

  // === Phase 3: check コマンド（成功） ===
  const verifyCommandStart = 220;
  const verifyOutputStart = 265;

  const verifyOutputLines: { segments: TextSegment[]; delay: number }[] = [
    {
      segments: [
        { text: "✓ ", color: "green", bold: true },
        { text: "query.sql", color: "white", bold: true },
        { text: " - ", color: "gray" },
        { text: "OK", color: "green", bold: true },
      ],
      delay: 0,
    },
    { segments: [{ text: "" }], delay: 5 },
    {
      segments: [
        { text: "Total: ", color: "gray" },
        { text: "1", color: "white", bold: true },
        { text: " file(s), ", color: "gray" },
        { text: "0", color: "green", bold: true },
        { text: " error(s)", color: "gray" },
      ],
      delay: 10,
    },
  ];

  return (
    <AbsoluteFill
      style={{
        backgroundColor: "#0d1117",
        padding: 30,
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <Terminal>
        {/* Phase 1: check コマンド（エラー） */}
        <TypingLine
          text={checkCommand}
          startFrame={0}
          typingSpeed={0.7}
          prefix={promptPrefix}
        />

        {frame >= checkOutputStart && (
          <>
            {checkOutputLines.map((line, i) => (
              <TerminalLine
                key={`check-${i}`}
                segments={line.segments}
                showAtFrame={checkOutputStart + line.delay}
              />
            ))}
          </>
        )}

        {/* Phase 2: fix コマンド */}
        {frame >= fixCommandStart && (
          <>
            <TerminalLine
              segments={[{ text: "" }]}
              showAtFrame={fixCommandStart}
            />
            <TypingLine
              text={fixCommand}
              startFrame={fixCommandStart + 5}
              typingSpeed={0.7}
              prefix={promptPrefix}
            />
          </>
        )}

        {frame >= fixOutputStart && (
          <>
            {fixOutputLines.map((line, i) => (
              <TerminalLine
                key={`fix-${i}`}
                segments={line.segments}
                showAtFrame={fixOutputStart + line.delay}
              />
            ))}
          </>
        )}

        {/* Phase 3: check コマンド（成功確認） */}
        {frame >= verifyCommandStart && (
          <>
            <TerminalLine
              segments={[{ text: "" }]}
              showAtFrame={verifyCommandStart}
            />
            <TypingLine
              text={checkCommand}
              startFrame={verifyCommandStart + 5}
              typingSpeed={0.7}
              prefix={promptPrefix}
            />
          </>
        )}

        {frame >= verifyOutputStart && (
          <>
            {verifyOutputLines.map((line, i) => (
              <TerminalLine
                key={`verify-${i}`}
                segments={line.segments}
                showAtFrame={verifyOutputStart + line.delay}
              />
            ))}
          </>
        )}
      </Terminal>
    </AbsoluteFill>
  );
};
