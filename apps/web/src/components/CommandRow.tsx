import CopyButton from "./CopyButton";

export default function CommandRow({
  label,
  command,
}: {
  label: string;
  command: string;
}) {
  return (
    <div className="cmd">
      <div className="cmd-head">
        <span>{label}</span>
        <CopyButton text={command} />
      </div>
      <code>{command}</code>
    </div>
  );
}
