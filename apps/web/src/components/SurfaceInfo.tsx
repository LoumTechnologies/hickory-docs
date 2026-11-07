import { useUiReading } from "../lib/uiReading";

export function SurfaceInfo({ name, editable, destination }: {
  name: string; editable: boolean; destination: string;
}) {
  useUiReading("surface identity", { name, editable, destination });
  return null;
}
