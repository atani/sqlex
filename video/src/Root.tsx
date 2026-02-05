import { Composition } from "remotion";
import { SqlexDemo } from "./SqlexDemo";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="SqlexDemo"
        component={SqlexDemo}
        durationInFrames={330}
        fps={30}
        width={800}
        height={500}
      />
    </>
  );
};
