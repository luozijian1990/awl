import { render } from "preact";
import { App } from "./app";
import { loadEntries, loadCalendars } from "./store";
import "./theme.css";

const root = document.getElementById("app");
if (root) render(<App />, root);

// 启动即拉取默认视图（draft inbox）的数据，以及侧栏/过滤用的日历组列表。
void loadEntries();
void loadCalendars();
