use svg::node::element::path::Data;
use svg::node::element::Polyline;
use svg::Document;
use svg::node::element::Path;
use core::marker::PhantomData;

struct Wrapper<'a,I:Iterator<Item=[f32;2]>+Clone+'a>(Option<I>,PhantomData<&'a I>);

impl<'a,I:Iterator<Item=[f32;2]>+Clone+'a> PlotTrait<'a> for Wrapper<'a,I>{
    #[inline(always)]
    fn ref_iter(&self)->Box<dyn Iterator<Item=[f32;2]>+'a>{
        Box::new(self.0.as_ref().unwrap().clone())
    }

    #[inline(always)]
    fn into_iter(&mut self)->Box<dyn Iterator<Item=[f32;2]>+'a>{
        Box::new(self.0.take().unwrap())
    }
}


trait PlotTrait<'a>{
    fn ref_iter(&self)->Box<dyn Iterator<Item=[f32;2]>+'a>;
    fn into_iter(&mut self)->Box<dyn Iterator<Item=[f32;2]>+'a>;
}

/*
enum Plot<'a>{
    Lines{name:String,plots:Box<dyn PlotTrait<'a>+'a>},
}
*/
struct Plot<'a>{
    name:String,
    plots:Box<dyn PlotTrait<'a>+'a>,
    //type? line/histo/scatter
}

pub struct Splot<'a>{
    title:String,
    xname:String,
    yname:String,
    plots:Vec<Plot<'a>>,
}


pub struct Color{
    pub back:u32,
    pub fore:u32,
    plots:[u32;10]
}

pub const DEFAULT_COLOR:Color=Color{
    back:0,
    fore:0,
    plots:[0;10]
};

impl<'a> Splot<'a>{
    pub fn new(title:impl ToString,xname:impl ToString,yname:impl ToString)->Splot<'a>{
        Splot{title:title.to_string(),plots:Vec::new(),xname:xname.to_string(),yname:yname.to_string()}
    }
    ///iterator will be iterated through twice by doing one call to clone().
    ///once to find min max bounds, second to construct plot
    pub fn lines<I:Iterator<Item=[f32;2]>+Clone+'a>(&mut self,name:impl ToString,plots:I)
    {
        self.plots.push(Plot{name:name.to_string(),plots:Box::new(Wrapper(Some(plots),PhantomData))})
    }

    pub fn render(mut self){
        let width=800.0;
        let height=600.0;
        let padding=150.0;
        
        let mut document = Document::new()
        .set("width",width)
        .set("height",height)
        .set("viewBox", (0,0, width, height));

        use svg::node::element::Rectangle;
        
        document=document.add(
            Rectangle::new()
            .set("fill","#e1e1db")
            .set("x","0")
            .set("y","0")
            .set("width",format!("{}",width))
            .set("height",format!("{}",height))
        );

        let data=svg::node::Text::new(format!("{}",self.title));
        let k=svg::node::element::Text::new().add(data).set("x",format!("{}",width/2.0)).set("y",format!("{}",padding/4.0)); 
        let k=k.set("alignment-baseline","start").set("text-anchor","middle").set("font-family","Arial");
        let k=k.set("font-size","x-large");
        document=document.add(k);

        let data=svg::node::Text::new(format!("X:  {}",self.xname));
        let k=svg::node::element::Text::new().add(data).set("x",format!("{}",width/2.0)).set("y",format!("{}",padding/2.0)); 
        let k=k.set("alignment-baseline","start").set("text-anchor","middle").set("font-family","Arial");
        let k=k.set("font-size","large");
        document=document.add(k);


        let data=svg::node::Text::new(format!("Y:  {}",self.yname));
        let k=svg::node::element::Text::new().add(data).set("x",format!("{}",width/2.0)).set("y",format!("{}",padding/1.5)); 
        let k=k.set("alignment-baseline","start").set("text-anchor","middle").set("font-family","Arial");
        let k=k.set("font-size","large");
        document=document.add(k);



        let [minx,maxx,miny,maxy]=if let Some(m)=find_bounds(self.plots.iter().flat_map(|a|a.plots.ref_iter())){
            m
        }else{
            return;
        };

        
        let scalex=(width-padding*2.0)/(maxx-minx);
        let scaley=(height-padding*2.0)/(maxy-miny);

        dbg!(minx,maxx,miny,maxy,scalex,scaley);
        

        //https://stackoverflow.com/questions/60497397/how-do-you-format-a-float-to-the-first-significant-decimal-and-with-specified-pr
        {
            let num_steps=10;
            let texty_padding=padding*0.2;
            let textx_padding=padding*0.4;
            
            let (xstep_num,xstep_power,xstep)=find_good_step(num_steps,(maxx-minx));
            let (ystep_num,ystep_power,ystep)=find_good_step(num_steps,(maxy-miny));
            dbg!(xstep,xstep_num,ystep,ystep_num,xstep_power,ystep_power);

            
            for a in 0..xstep_num{
                let p=(a as f32)*xstep;
                let precision=1+xstep_power as usize;
                let data=svg::node::Text::new(format!("{0:.1$}",p,precision));
                let k=svg::node::element::Text::new().add(data).set("x",format!("{}",p*scalex+padding)).set("y",format!("{}",height-padding+textx_padding)); 
                let k=k.set("alignment-baseline","start").set("text-anchor","middle").set("font-family","Arial");                
                document=document.add(k);
            }


            for a in 0..ystep_num{
                let p=(a as f32)*ystep;
                let precision=1+ystep_power as usize;
                let data=svg::node::Text::new(format!("{0:.1$}",p,precision));
                let k=svg::node::element::Text::new().add(data).set("x",format!("{}",padding-texty_padding)).set("y",format!("{}",height-p*scaley-padding)); 
                let k=k.set("alignment-baseline","middle").set("text-anchor","end").set("font-family","Arial");
                document=document.add(k);
            }

        }

        let data = Data::new()
        .move_to((padding, padding))
        .line_to((padding,height-padding))
        .line_to((width-padding,height-padding));
        
        let vert_line = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3)
        .set("d", data);
        
        document=document.add(vert_line);

        for Plot{name,mut plots} in self.plots.into_iter(){
            
            let mut data=Polyline::new().set("fill","none").set("stroke","#0074d9").set("stroke-width",3);
            
            let mut it=plots.into_iter();

            use std::fmt::Write;
            let mut points=String::new();
            if let Some([x,y])=it.next(){
                for [x,y] in it{
                    write!(&mut points,"{},{}\n",padding+x*scalex,height-padding-y*scaley);
                }   
            }
            data=data.set("points",points);
            document=document.add(data);    
        }

        svg::save("image.svg", &document).unwrap();
    
    }
}


fn find_good_step(num_steps:usize,range:f32)->(usize,f32,f32){
    //https://stackoverflow.com/questions/237220/tickmark-algorithm-for-a-graph-axis
    

    let rough_step=range/(num_steps-1) as f32;
    
    let step_power=10.0f32.powf(-rough_step.abs().log10().floor()) as f32;
    let normalized_step=rough_step*step_power;
    dbg!(normalized_step);

    let good_steps=[1.0,2.0,5.0,10.0];
    let good_normalized_step=good_steps.iter().find(|a|**a>normalized_step).unwrap();
    dbg!(good_normalized_step);


    let step=good_normalized_step/ step_power;


    let new_step=if range%step!=0.0{
        (range/step) as usize+1
    }else{
        (range/step) as usize
    };
    
    (new_step+1,step_power.log10(),step)
}


fn main() {
    dbg!(find_good_step(10,0.15));
    dbg!(find_good_step(10,2.15));
    dbg!(find_good_step(10,12556.15));
    dbg!(find_good_step(10,5467.0));


    let mut s=Splot::new("Testing testing one two three","this is x","this is y");
    //s.lines("yo", (0..50).map(|x|x as f32).map(|x|x*0.5).map(|x|[x,x.sin()+1.0]) );
    s.lines("yo", (0..500).map(|x|x as f32).map(|x|x).map(|x|[x*0.000002,x*0.000002]) );
    
    s.render();
    

}



fn find_bounds(mut it:impl IntoIterator<Item=[f32;2]>)->Option<[f32;4]>{
    let mut ii=it.into_iter();
    if let Some([x,y])=ii.next(){
        let mut val=[x,x,y,y];
        ii.fold(&mut val,|val,[x,y]|{
            if x<val[0]{
                val[0]=x;
            }else if x>val[1]{
                val[1]=x;
            }
            if y<val[2]{
                val[2]=y;
            }else if y>val[3]{
                val[3]=y;
            }
            val
        });
        Some(val)
    }else{
        None
    }
}