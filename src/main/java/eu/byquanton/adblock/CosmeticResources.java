package eu.byquanton.adblock;

import java.util.Set;
import java.util.stream.Collectors;


/**
 * @param hideSelectors  CSS selectors that should be hidden via display: none !important
 * @param injectedScript Scripts that should be injected into the page (scriptlets)
 * @param exceptions     Selectors that should be excluded from generic element hiding rules
 * @param genericHide    Whether generic element hiding rules should be disabled for this page
 */
public record CosmeticResources(Set<String> hideSelectors, String injectedScript, Set<String> exceptions,
                                boolean genericHide) {

    /**
     * Convert all hide selectors into a single CSS stylesheet.
     *
     * @return A CSS stylesheet containing display:none rules for all selectors
     */
    public String selectorsToStylesheet() {
        if (hideSelectors.isEmpty()) {
            return "";
        }
        return hideSelectors.stream()
                .map(selector -> selector + " { display: none !important; }\n")
                .collect(Collectors.joining());
    }
}