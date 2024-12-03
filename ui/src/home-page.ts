import { AppClient } from "@holochain/client";
import { consume } from "@lit/context";
import { msg } from "@lit/localize";
import { appClientContext, Router, Routes } from "@tnesh-stack/elements";
import { AsyncResult, SignalWatcher } from "@tnesh-stack/signals";
import { EntryRecord } from "@tnesh-stack/utils";
import { css, html, LitElement } from "lit";
import { customElement } from "lit/decorators.js";

import "@tnesh-stack/elements/dist/elements/display-error.js";
import "@darksoil-studio/profiles-zome/dist/elements/profile-list-item.js";

import { appStyles } from "./app-styles.js";

@customElement("home-page")
export class HomePage extends SignalWatcher(LitElement) {
  @consume({ context: appClientContext })
  client!: AppClient;

  renderContent() {
    return html`
      <span>TODO: replace this with the content of your app.</span>
      <span>Maybe you want to import elements from one of the TNESH modules?</span>
    `;
  }

  render() {
    return html`
      <div class="column" style="flex: 1">
        <div class="row top-bar">
          <span class="title" style="flex: 1">${msg("Messenger Demo")}</span>

          <div class="row" style="gap: 16px">
            <profile-list-item 
              @click=${() =>
      this.dispatchEvent(
        new CustomEvent("profile-clicked", {
          detail: true,
          composed: true,
        }),
      )}
              .agentPubKey=${this.client.myPubKey}
            ></profile-list-item>
          </div>
        </div>

        <div class="column" style="flex: 1; align-items: center; justify-content: center;">
          ${this.renderContent()}
        </div>
      </div>
    `;
  }

  static styles = [
    css`
      :host {
        display: flex;
        flex: 1;
      }

    `,
    ...appStyles,
  ];
}
