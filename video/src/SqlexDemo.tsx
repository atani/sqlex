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

  // コマンド入力（フレーム 0-60）
  const command = "sqlex check query.sql";
  const commandEndFrame = 60;
  const outputStartFrame = 75;

  // 出力行（フレームごとにフェードイン）
  const outputLines: { segments: TextSegment[]; delay: number }[] = [
    // エラーヘッダー
    {
      segments: [
        { text: "✗ ", color: "red", bold: true },
        { text: "query.sql", color: "white", bold: true },
        { text: " - ", color: "gray" },
        { text: "1 error(s)", color: "red" },
      ],
      delay: 0,
    },
    // 空行
    { segments: [{ text: "" }], delay: 5 },
    // エラーメッセージ
    {
      segments: [
        { text: "  Syntax error ", color: "red" },
        { text: "(line 4, col 6)", color: "gray" },
        { text: ": Expected expression, found: ", color: "white" },
        { text: "FROM", color: "cyan", bold: true },
      ],
      delay: 10,
    },
    // ヒント
    {
      segments: [
        { text: "  💡 ", color: "yellow" },
        { text: "Line 3 may have a trailing comma that should be removed", color: "yellow" },
      ],
      delay: 20,
    },
    // 空行
    { segments: [{ text: "" }], delay: 25 },
    // コード行 2
    {
      segments: [
        { text: "  2 ", color: "gray" },
        { text: "│ ", color: "gray" },
        { text: "  name,", color: "white" },
      ],
      delay: 30,
    },
    // コード行 3 (問題のある行)
    {
      segments: [
        { text: "  3 ", color: "yellow", bold: true },
        { text: "│ ", color: "gray" },
        { text: "  email,", color: "white" },
        { text: "  ← check here", color: "yellow" },
      ],
      delay: 35,
    },
    // コード行 4 (エラー行)
    {
      segments: [
        { text: "  4 ", color: "red", bold: true },
        { text: "│ ", color: "gray" },
        { text: "FROM", color: "cyan", bold: true },
        { text: " users", color: "white" },
      ],
      delay: 40,
    },
    // エラー位置マーカー
    {
      segments: [
        { text: "    ", color: "gray" },
        { text: "│ ", color: "gray" },
        { text: "     ", color: "white" },
        { text: "^", color: "red", bold: true },
      ],
      delay: 45,
    },
    // コード行 5
    {
      segments: [
        { text: "  5 ", color: "gray" },
        { text: "│ ", color: "gray" },
        { text: "WHERE active = 1", color: "white" },
      ],
      delay: 50,
    },
    // 空行
    { segments: [{ text: "" }], delay: 55 },
    // サマリー
    {
      segments: [
        { text: "Total: ", color: "gray" },
        { text: "1", color: "white", bold: true },
        { text: " file(s), ", color: "gray" },
        { text: "1", color: "red", bold: true },
        { text: " error(s)", color: "gray" },
      ],
      delay: 65,
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
        {/* コマンド入力（タイピングアニメーション） */}
        <TypingLine
          text={command}
          startFrame={0}
          typingSpeed={0.6}
          prefix={promptPrefix}
        />

        {/* 出力行 */}
        {frame >= outputStartFrame && (
          <>
            {outputLines.map((line, i) => (
              <TerminalLine
                key={i}
                segments={line.segments}
                showAtFrame={outputStartFrame + line.delay}
              />
            ))}
          </>
        )}
      </Terminal>
    </AbsoluteFill>
  );
};
