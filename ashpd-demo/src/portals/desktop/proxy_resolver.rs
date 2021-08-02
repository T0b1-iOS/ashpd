use adw::prelude::*;
use ashpd::{desktop::proxy_resolver::ProxyResolverProxy, zbus};
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/proxy_resolver.ui")]
    pub struct ProxyResolverPage {
        #[template_child]
        pub uri: TemplateChild<gtk::Entry>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProxyResolverPage {
        const NAME: &'static str = "ProxyResolverPage";
        type Type = super::ProxyResolverPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<adw::ClampLayout>();
            klass.install_action(
                "proxy_resolver.resolve",
                None,
                move |page, _action, _target| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(@weak page => async move {
                        if let Err(err) = page.resolve().await {
                            tracing::error!("Failed to resolve proxy {}", err);
                        }
                    }));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for ProxyResolverPage {}
    impl WidgetImpl for ProxyResolverPage {}
    impl BinImpl for ProxyResolverPage {}
}

glib::wrapper! {
    pub struct ProxyResolverPage(ObjectSubclass<imp::ProxyResolverPage>) @extends gtk::Widget, adw::Bin;
}

impl ProxyResolverPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a ProxyResolverPage")
    }

    async fn resolve(&self) -> ashpd::Result<()> {
        let self_ = imp::ProxyResolverPage::from_instance(self);
        let uri = self_.uri.text();

        let cnx = zbus::azync::Connection::session().await?;
        let proxy = ProxyResolverProxy::new(&cnx).await?;

        let resolved_uris = proxy.lookup(&uri).await?;

        resolved_uris.iter().for_each(|uri| {
            let row = adw::ActionRow::builder().title(uri).build();
            self_.response_group.add(&row);
        });

        self_.response_group.show();
        Ok(())
    }
}
