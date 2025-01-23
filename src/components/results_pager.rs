use std::io::BufRead;

#[derive(Debug, Clone)]
pub struct ResultsPager {
    pub results_per_page: usize,
    pub page_idx: usize,
    pub num_pages: usize,
    pub total_results: usize,
    pub paged_item: Vec<String>,
    pub remainder: usize,
}

impl Default for ResultsPager {
    fn default() -> Self {
        Self {
            results_per_page: 20,
            page_idx: 0,
            num_pages: 0,
            total_results: 0,
            paged_item: Vec::new(),
            remainder: 0,
        }
    }
}

impl ResultsPager {
    pub fn init(&mut self, results: &Vec<u8>, selection: Vec<String>) {
        // take results and see how many there are
        let num_results = results.lines().count();

        self.total_results = num_results;
        self.paged_item = selection;

        match (
            num_results / self.results_per_page,
            num_results % self.results_per_page,
        ) {
            // if less than one page, make 1 page
            (div, rem) if div < 1 => {
                self.num_pages = 1;
                self.remainder = rem;
            }
            // if remainder > 0, show it
            (div, rem) if rem > 0 => {
                self.num_pages = div + 1;
                self.remainder = rem;
            }
            (div, rem) => {
                self.num_pages = div;
                self.remainder = rem;
            }
        }
    }
    pub fn set_page_idx(&mut self, new_page_idx: usize) {
        self.page_idx = new_page_idx;
    }
}
