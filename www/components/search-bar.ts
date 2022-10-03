import { LitElement, html, css } from 'lit';
import { map } from 'lit/directives/map';
import { customElement, state } from 'lit/decorators';

/** Expected output from complete API */
interface AutocompleteOption {
    subject: string[],
    code: string[],
    section: string[],
    semester: string[],
    year: string[],
    professor: string[]
}

@customElement("search-bar")
export class SearchBar extends LitElement {

    static override styles = css`
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

    @state()
    protected autocomplete_options = [];

    input_field() {
        return html`<input @input="${this.fetch_autocomplete_options}" id="query" name="query" list="ac-datalist" type="search" autocomplete="off" value="">`
    }


    key_option(option: AutocompleteOption) {
        return `${option.subject[0]} ${option.code[0]}.${option.section[0]} ${option.semester[0]} ${option.year[0]}`
    }

    display_option(option: AutocompleteOption) {
        return `${this.key_option(option)} ${option.professor.filter((p: string) => p.length > 0).join("; ")}`
    }


    options() {
        return map(this.autocomplete_options, o => html`<option value="${this.key_option(o)}">${this.display_option(o)}</option>`)
    }

    datalist() {
        return html`<datalist id="ac-datalist">${this.options()}</datalist>`
    }

    override render() {
        return html`
        ${this.input_field()}
        ${this.datalist()}
        `
    }

    fetch_autocomplete_options(event?: InputEvent) {
        const input = event?.target as HTMLInputElement;
        const query = input?.value;
        if (query) {
            fetch(`/complete?q=${encodeURIComponent(query)}`)
                .then(res => {
                    console.debug(`completion of "${query}" took ${res.headers.get("x-response-time")}`)
                    return res.json()
                })
                .then(res => this.autocomplete_options = res)
                .catch(e => console.error(e))
        }
    }

}
