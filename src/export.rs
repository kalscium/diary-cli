use std::path::Path;
use crate::{entry::{Entry, Section}, Scribe, scribe_write, archive::Archive, search};
use soulog::*;

pub fn export_md(strict: bool, tags: Option<Vec<String>>, path: String, mut logger: impl Logger) {
    log!((logger) Export("Exporting archive to path '{path}'..."));
    let archive = Archive::load(logger.hollow());

    // Get entries and mocs
    let mut entries = match tags {
        Some(x) => 
            (if strict { search::search(&x, archive.list_entries(logger.hollow()), logger.hollow()) }
            else { search::search_strict(&x, archive.list_entries(logger.hollow()), logger.hollow()) })
                .into_iter().map(|x| archive.get_entry(x, logger.hollow()).unwrap()).collect(),
        None => archive.list_entries(logger.hollow()),
    }; // <copy for moc later>

    // Export em
    let path = Path::new(&path);
    entries.iter_mut().for_each(|x| export_entry(path, x, logger.hollow()));
}

pub fn export_entry(path: &Path, entry: &mut Entry, mut logger: impl Logger) {
    log!((logger) Export("Exporting entry of uid '{}'...", entry.uid));
    let mut scribe = Scribe::new(path.join(&entry.uid).with_extension("md"), logger.hollow());

    // Tags, title and description
    scribe_tags(entry.tags(logger.hollow()), &mut scribe);
    scribe_write!((scribe) "# ", entry.title(logger.hollow()), "\n");
    scribe.write_line("---");
    scribe_write!((scribe) "**Description:** ", entry.description(logger.hollow()), "\n");

    // Notes
    scribe.write_line("## Notes");
    entry.notes(logger.hollow()).iter().for_each(|x| scribe_write!((scribe) "- ", x, "\n"));    

    // Section's notes
    entry.sections(logger.hollow()).iter_mut().for_each(|section| {
        scribe_write!((scribe) "- #### ", section.title(logger.hollow()), "\n");
        section.notes(logger.hollow()).iter().for_each(|x| scribe_write!((scribe) "\t- ", x, "\n"));
        section.clear_cache();
    });
    scribe.write_line("---");

    // Sections
    entry.sections(logger.hollow()).iter_mut().for_each(|x| export_section_content(&mut scribe, x, logger.hollow()));

    entry.clear_cache();
}

fn export_section_content(scribe: &mut Scribe<impl Logger>, section: &mut Section, logger: impl Logger,) {
    scribe_write!((scribe) "### ", section.title(logger.hollow()), "\n");
    let content = section.content(logger.hollow()).trim_end_matches('\n').split("\n");
    content.for_each(|x| {
        scribe_write!((scribe) "> ", x, "\n");
    });
    section.clear_cache();
}

fn scribe_tags(tags: &[String], scribe: &mut Scribe<impl Logger>) {
    scribe.write_line("---");
    scribe.write("tags: diary-cli");
    tags.iter().for_each(|x| scribe_write!((scribe) ", ", x));
    scribe.new_line();
    scribe.write_line("---");
}