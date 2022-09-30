import { LitElement, html, css } from 'lit';
import { ref } from 'lit/directives/ref';
import { map } from 'lit/directives/map';

export class SearchBar extends LitElement {

	static styles = css`
		input {
			padding: 5px 0.5em;
			width: 75%;
		}

		@media (max-width: 375px) {
			input {
				width: 100%;
			}
		}
	`

	static properties = {
		autocomplete_options: { state: true, type: Array }
	}

	constructor() {
		super()
		this.autocomplete_options = []
	}

	input_field() {
		return html`<input ${ref(this.fetch_autocomplete_options)} @input="${this.fetch_autocomplete_options}"
            id="query" name="query" list="ac-datalist" type="search" autocomplete="off" value="">`
	}


	key_option(option) {
		return `${option.subject[0]} ${option.code[0]}.${option.section[0]} ${option.semester[0]} ${option.year[0]}`
	}

	display_option(option) {
		return `${this.key_option(option)} ${option.professor.filter(p => p.length > 0).join("; ")}`
	}


	options() {
		return map(this.autocomplete_options, o => html`<option value="${this.key_option(o)}">${this.display_option(o)}</option>`)
	}

	datalist() {
		return html`<datalist id="ac-datalist">${this.options()}</datalist>`
	}

	render() {
		return html`
        ${this.input_field()}
        ${this.datalist()}
        `
	}

	fetch_autocomplete_options(input) {
		let query;
		if (query = input?.target?.value) {
			fetch(`/complete?q="${encodeURIComponent(query)}"`)
				.then(res => {
					console.debug(`completion of "${query}" took ${res.headers.get("x-response-time")}`)
					return res.json()
				})
				.then(res => this.autocomplete_options = res)
				.catch(e => console.error(e))
		}
	}

}

customElements.define('search-bar', SearchBar)
