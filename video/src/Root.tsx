import { Composition } from "remotion";
import { SqlexDemo } from "./SqlexDemo";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="SqlexDemo"
        component={SqlexDemo}
        durationInFrames={300}
        fps={30}
        width={800}
        height={500}
      />
    </>
  );
};
