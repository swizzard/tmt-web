import { TabWithTags } from "../api";

export default function TabForm({ tab }: { tab?: TabWithTags }) {
  return (
    <>
      <label htmlFor="url">URL</label>
      <input type="url" id="url" name="url" defaultValue={tab?.tab.url} />
      <label htmlFor="notes">Notes</label>
      <textarea
        id="notes"
        name="notes"
        defaultValue={tab?.tab.notes}
        rows={10}
        cols={50}
      />
      <div className="tags">
        <label htmlFor="tags">Tags</label>
        <select id="tags" name="tags" multiple></select>
      </div>
    </>
  );
}
